package com.oloko.forecast.widget

import android.content.Context
import androidx.work.Constraints
import androidx.work.Data
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.Worker
import androidx.work.WorkerParameters
import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder
import java.util.concurrent.TimeUnit

/**
 * Keeps the widget fresh without opening the app: fetches the last location's
 * forecast from Open-Meteo and writes widget.json in the app's format.
 */
class RefreshWorker(context: Context, params: WorkerParameters) : Worker(context, params) {
  override fun doWork(): Result {
    val force = inputData.getBoolean(KEY_FORCE, false)
    try {
      val cached = Store.readForecast(applicationContext)
      // The app may have just refreshed; don't fetch again for a periodic run.
      val s = Settings.load(applicationContext)
      val minAge = s.refreshMinutes * 60_000L * 2 / 3
      if (!force && cached != null && System.currentTimeMillis() - cached.fetchedAt < minAge) return Result.success()
      val lat = s.lat ?: return Result.success()
      val lon = s.lon ?: return Result.success()
      val json = fetch(lat, lon, s.timezone ?: "auto", s.days)
      Store.writeJson(applicationContext, Store.WIDGET, toWidgetJson(json, Store.readJson(applicationContext, Store.CONFIG)))
      return Result.success()
    } catch (e: Exception) {
      return if (runAttemptCount < 2) Result.retry() else Result.failure()
    } finally {
      applicationContext.getSharedPreferences("forecast_widget", Context.MODE_PRIVATE)
        .edit().putBoolean(ForecastWidget.KEY_REFRESHING, false).apply()
      ForecastWidget.updateAll(applicationContext)
    }
  }

  private fun fetch(lat: Double, lon: Double, timezone: String, days: Int): JSONObject {
    val query = listOf(
      "latitude" to lat.toString(),
      "longitude" to lon.toString(),
      "timezone" to timezone,
      "daily" to "temperature_2m_max,temperature_2m_min,weather_code,precipitation_sum",
      "hourly" to "temperature_2m,weather_code,precipitation,precipitation_probability",
      "forecast_days" to days.coerceIn(1, 16).toString(),
    ).joinToString("&") { (k, v) -> "$k=${URLEncoder.encode(v, "UTF-8")}" }
    val connection = URL("https://api.open-meteo.com/v1/forecast?$query").openConnection() as HttpURLConnection
    connection.connectTimeout = 15_000
    connection.readTimeout = 15_000
    connection.setRequestProperty("User-Agent", "forecast-mobile-widget")
    try {
      if (connection.responseCode != 200) error("Open-Meteo HTTP ${connection.responseCode}")
      return JSONObject(connection.inputStream.bufferedReader().readText())
    } finally {
      connection.disconnect()
    }
  }

  /** Open-Meteo response → the widget.json shape the app (Rust) writes. */
  private fun toWidgetJson(api: JSONObject, config: JSONObject?): JSONObject {
    fun weather(code: Int) = Wmo.decode(code).let { (description, icon) ->
      JSONObject().put("description", description).put("icon", icon)
    }
    val h = api.getJSONObject("hourly")
    val hourly = JSONArray()
    val times = h.getJSONArray("time")
    for (i in 0 until times.length()) {
      hourly.put(
        JSONObject()
          .put("time", times.getString(i) + ":00")
          .put("temperature", h.getJSONArray("temperature_2m").getDouble(i))
          .put("precipitation", h.getJSONArray("precipitation").optDouble(i, 0.0))
          .put("weather", weather(h.getJSONArray("weather_code").optInt(i, -1))),
      )
    }
    val d = api.getJSONObject("daily")
    val daily = JSONArray()
    val dates = d.getJSONArray("time")
    for (i in 0 until dates.length()) {
      daily.put(
        JSONObject()
          .put("date", dates.getString(i))
          .put("temperature_max", d.getJSONArray("temperature_2m_max").getDouble(i))
          .put("temperature_min", d.getJSONArray("temperature_2m_min").getDouble(i))
          .put("precipitation", d.getJSONArray("precipitation_sum").optDouble(i, 0.0))
          .put("weather", weather(d.getJSONArray("weather_code").optInt(i, -1))),
      )
    }
    val forecast = JSONObject()
      .put("source", "open_meteo")
      .put("fallback", false)
      .put("utc_offset_seconds", api.optInt("utc_offset_seconds", 0))
      .put("timezone", api.optString("timezone"))
      .put("daily", daily)
      .put("hourly", hourly)
    return JSONObject()
      .put("location", config?.optJSONObject("last_location") ?: JSONObject())
      .put("forecast", forecast)
      .put("fetched_at", System.currentTimeMillis())
  }

  companion object {
    private const val PERIODIC = "forecast-widget-refresh"
    private const val NOW = "forecast-widget-refresh-now"
    private const val KEY_FORCE = "force"

    private val network = Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build()

    /** Runs every [minutes] (Android's minimum is 15); UPDATE keeps the schedule when unchanged. */
    fun schedule(context: Context, minutes: Int) {
      val request = PeriodicWorkRequestBuilder<RefreshWorker>(minutes.coerceAtLeast(15).toLong(), TimeUnit.MINUTES)
        .setConstraints(network)
        .build()
      WorkManager.getInstance(context).enqueueUniquePeriodicWork(PERIODIC, ExistingPeriodicWorkPolicy.UPDATE, request)
    }

    fun refreshNow(context: Context) {
      val request = OneTimeWorkRequestBuilder<RefreshWorker>()
        .setConstraints(network)
        .setInputData(Data.Builder().putBoolean(KEY_FORCE, true).build())
        .build()
      WorkManager.getInstance(context).enqueueUniqueWork(NOW, ExistingWorkPolicy.REPLACE, request)
    }

    fun cancel(context: Context) {
      WorkManager.getInstance(context).cancelUniqueWork(PERIODIC)
    }
  }
}
