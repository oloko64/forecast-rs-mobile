package com.oloko.forecast.widget

import android.content.Context
import android.content.res.Configuration
import android.graphics.Color
import org.json.JSONObject
import java.io.File
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale
import java.util.TimeZone

/** One hour of the forecast, as written by the app (Rust) or the widget's own refresh. */
data class Hour(val time: String, val temperature: Double, val precipitation: Double, val description: String, val icon: String) {
  val date: String get() = time.take(10)
  val hourKey: String get() = time.take(13)
}

data class Day(val date: String, val max: Double, val min: Double, val precipitation: Double)

data class Forecast(
  val locationName: String,
  val utcOffsetSeconds: Int,
  val fetchedAt: Long,
  val hours: List<Hour>,
  val days: List<Day>,
) {
  /** "yyyy-MM-ddTHH" of the current hour at the location. */
  fun nowKey(now: Long = System.currentTimeMillis()): String {
    val fmt = SimpleDateFormat("yyyy-MM-dd'T'HH", Locale.US).apply { timeZone = TimeZone.getTimeZone("UTC") }
    return fmt.format(Date(now + utcOffsetSeconds * 1000L))
  }
}

/** What the widget shows and how (config.json "widget"); defaults match the app's. */
data class WidgetShow(
  val location: Boolean = false,
  val now: Boolean = true,
  val summary: Boolean = false,
  val dayPicker: Boolean = true,
  val refreshButton: Boolean = true,
  val rain: Boolean = true,
  /** "off", "small" or "big". */
  val clock: String = "big",
  /** Background opacity in percent. */
  val background: Int = 100,
) {
  /** Under half opaque: white text with shadows, to read over the wallpaper. */
  val transparent: Boolean get() = background < 50
}

/** The app's settings that matter to the widget (from config.json). */
data class Settings(
  val dark: Boolean,
  val card: Int,
  val fg: Int,
  val muted: Int,
  val line: Int,
  val accent: Int,
  val temp: Int,
  val rain: Int,
  val fahrenheit: Boolean,
  val portuguese: Boolean,
  val days: Int,
  val lat: Double?,
  val lon: Double?,
  val timezone: String?,
  val locationName: String?,
  val show: WidgetShow,
  val refreshMinutes: Int,
) {
  fun toUnit(celsius: Double) = if (fahrenheit) celsius * 9 / 5 + 32 else celsius
  fun temp(celsius: Double) = "${Math.round(toUnit(celsius))}°"
  fun tempUnit(celsius: Double) = "${temp(celsius)}${if (fahrenheit) "F" else "C"}"
  fun mm(value: Double): String {
    val text = if (value >= 10) String.format(Locale.US, "%.0f", value) else String.format(Locale.US, "%.1f", value)
    return if (portuguese) text.replace('.', ',') else text
  }
  fun text(en: String, pt: String) = if (portuguese) pt else en
  fun weather(description: String) = if (portuguese) WEATHER_PT[description] ?: description else description

  /** "Today", "Sat 28" / "Hoje", "Sáb 28". */
  fun dayLabel(date: String, today: String): String {
    if (date == today) return text("Today", "Hoje")
    val cal = Calendar.getInstance(TimeZone.getTimeZone("UTC"), Locale.US)
    cal.time = SimpleDateFormat("yyyy-MM-dd", Locale.US).apply { timeZone = TimeZone.getTimeZone("UTC") }.parse(date) ?: return date
    val names = if (portuguese) WEEKDAYS_PT else WEEKDAYS_EN
    return "${names[cal.get(Calendar.DAY_OF_WEEK) - 1]} ${cal.get(Calendar.DAY_OF_MONTH)}"
  }

  companion object {
    private val WEEKDAYS_EN = listOf("Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat")
    private val WEEKDAYS_PT = listOf("Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb")

    /** Light/dark preset colors, matching src/app.css. */
    private val PALETTES = mapOf(
      "ocean" to listOf("#3867f0", "#f08a24", "#2f8ff5", "#6b8cff", "#ffa14a", "#5aa9ff"),
      "forest" to listOf("#23845a", "#d4900c", "#1f8aa3", "#5cc98d", "#f2c14e", "#4fc3d9"),
      "sunset" to listOf("#c2417a", "#ea5f2b", "#6a5acd", "#ff7eb3", "#ff8f66", "#a597ff"),
    )

    fun load(context: Context): Settings {
      val cfg = Store.readJson(context, Store.CONFIG) ?: JSONObject()
      val dark = when (cfg.optString("theme_mode", "system")) {
        "dark" -> true
        "light" -> false
        else -> (context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES
      }
      val paletteId = cfg.optString("palette", "ocean").let { if (it in PALETTES) it else "ocean" }
      val preset = PALETTES.getValue(paletteId).let { if (dark) it.drop(3) else it.take(3) }
      val custom = cfg.optJSONObject("custom_colors")?.optJSONObject(paletteId)
      fun color(role: String, fallback: String): Int {
        val hex = custom?.optString(role).orEmpty()
        return runCatching { Color.parseColor(hex.ifEmpty { fallback }) }.getOrDefault(Color.parseColor(fallback))
      }
      val language = cfg.optString("language", "system")
      val portuguese = when (language) {
        "pt-BR" -> true
        "en" -> false
        else -> Locale.getDefault().language == "pt"
      }
      val location = cfg.optJSONObject("last_location")
      val w = cfg.optJSONObject("widget")
      val defaults = WidgetShow()
      fun shown(key: String, default: Boolean) = w?.optBoolean(key, default) ?: default
      val show = WidgetShow(
        location = shown("location", defaults.location),
        now = shown("now", defaults.now),
        summary = shown("summary", defaults.summary),
        dayPicker = shown("day_picker", defaults.dayPicker),
        refreshButton = shown("refresh_button", defaults.refreshButton),
        rain = shown("rain", defaults.rain),
        clock = w?.optString("clock", defaults.clock)?.takeIf { it in setOf("off", "small", "big") } ?: defaults.clock,
        background = (w?.optInt("background", defaults.background) ?: defaults.background).coerceIn(0, 100),
      )
      val white = Color.WHITE
      return Settings(
        dark = dark,
        // See-through widgets get a dark tint in either theme, so the white text stays readable.
        card = Color.parseColor(if (dark || show.transparent) "#182030" else "#ffffff"),
        fg = if (show.transparent) white else Color.parseColor(if (dark) "#e9eef7" else "#172033"),
        muted = if (show.transparent) Color.parseColor("#E6FFFFFF") else Color.parseColor(if (dark) "#8e9ab0" else "#66728a"),
        line = if (show.transparent) Color.parseColor("#66FFFFFF") else Color.parseColor(if (dark) "#263147" else "#e1e6ef"),
        accent = color("accent", preset[0]),
        temp = color("temp", preset[1]),
        rain = color("rain", preset[2]),
        fahrenheit = cfg.optString("units") == "fahrenheit",
        portuguese = portuguese,
        days = cfg.optInt("days", 7).coerceIn(1, 16),
        lat = location?.optDouble("lat")?.takeUnless { it.isNaN() },
        lon = location?.optDouble("lon")?.takeUnless { it.isNaN() },
        timezone = location?.optString("timezone")?.ifEmpty { null },
        locationName = location?.optString("name")?.ifEmpty { null },
        show = show,
        refreshMinutes = cfg.optInt("widget_refresh_minutes", 30).coerceIn(15, 240),
      )
    }

    private val WEATHER_PT = mapOf(
      "Clear sky" to "Céu limpo", "Mainly clear" to "Predominantemente limpo",
      "Partly cloudy" to "Parcialmente nublado", "Overcast" to "Nublado", "Fog" to "Neblina",
      "Drizzle" to "Garoa", "Freezing drizzle" to "Garoa congelante", "Slight rain" to "Chuva fraca",
      "Moderate rain" to "Chuva moderada", "Heavy rain" to "Chuva forte", "Freezing rain" to "Chuva congelante",
      "Slight snow fall" to "Neve fraca", "Moderate snow fall" to "Neve moderada", "Heavy snow fall" to "Neve forte",
      "Snow grains" to "Grãos de neve", "Rain showers" to "Pancadas de chuva", "Snow showers" to "Pancadas de neve",
      "Thunderstorm" to "Tempestade", "Thunderstorm with hail" to "Tempestade com granizo", "Unknown" to "Desconhecido",
    )
  }
}

/** Files shared with the app, in its data directory (Tauri's app_data_dir on Android). */
object Store {
  const val CONFIG = "config.json"
  const val WIDGET = "widget.json"

  private fun file(context: Context, name: String) = File(context.dataDir, name)

  fun readJson(context: Context, name: String): JSONObject? =
    runCatching { JSONObject(file(context, name).readText()) }.getOrNull()

  /** Atomic write, like the app's: a reader never sees half a file. */
  fun writeJson(context: Context, name: String, json: JSONObject) {
    val target = file(context, name)
    val tmp = File(target.parentFile, "$name.widget.tmp")
    tmp.writeText(json.toString())
    if (!tmp.renameTo(target)) {
      target.writeText(json.toString())
      tmp.delete()
    }
  }

  fun readForecast(context: Context): Forecast? {
    val root = readJson(context, WIDGET) ?: return null
    val forecast = root.optJSONObject("forecast") ?: return null
    val hourly = forecast.optJSONArray("hourly") ?: return null
    val daily = forecast.optJSONArray("daily")
    val hours = (0 until hourly.length()).map { i ->
      val h = hourly.getJSONObject(i)
      val w = h.optJSONObject("weather")
      Hour(
        time = h.getString("time"),
        temperature = h.getDouble("temperature"),
        precipitation = h.optDouble("precipitation", 0.0),
        description = w?.optString("description").orEmpty(),
        icon = w?.optString("icon").orEmpty(),
      )
    }
    val days = (0 until (daily?.length() ?: 0)).map { i ->
      val d = daily!!.getJSONObject(i)
      Day(d.getString("date"), d.getDouble("temperature_max"), d.getDouble("temperature_min"), d.optDouble("precipitation", 0.0))
    }
    return Forecast(
      locationName = root.optJSONObject("location")?.optString("name").orEmpty(),
      utcOffsetSeconds = forecast.optInt("utc_offset_seconds", 0),
      fetchedAt = root.optLong("fetched_at", 0),
      hours = hours,
      days = days,
    )
  }
}

/** WMO weather codes, as decoded by the app (src-tauri/src/openmeteo.rs). */
object Wmo {
  fun decode(code: Int): Pair<String, String> = when (code) {
    0 -> "Clear sky" to "☀️"
    1 -> "Mainly clear" to "🌤️"
    2 -> "Partly cloudy" to "⛅"
    3 -> "Overcast" to "☁️"
    45, 48 -> "Fog" to "🌫️"
    51, 53, 55 -> "Drizzle" to "🌦️"
    56, 57 -> "Freezing drizzle" to "🥶"
    61 -> "Slight rain" to "🌧️"
    63 -> "Moderate rain" to "🌧️"
    65 -> "Heavy rain" to "🌧️"
    66, 67 -> "Freezing rain" to "🧊"
    71 -> "Slight snow fall" to "🌨️"
    73 -> "Moderate snow fall" to "🌨️"
    75 -> "Heavy snow fall" to "❄️"
    77 -> "Snow grains" to "🌨️"
    80, 81, 82 -> "Rain showers" to "🌦️"
    85, 86 -> "Snow showers" to "🌨️"
    95 -> "Thunderstorm" to "⛈️"
    96, 99 -> "Thunderstorm with hail" to "⛈️"
    else -> "Unknown" to "❔"
  }
}
