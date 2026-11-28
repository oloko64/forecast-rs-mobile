package com.oloko.forecast.widget

import android.app.AlarmManager
import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.provider.AlarmClock
import android.view.View
import android.widget.RemoteViews
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.Worker
import androidx.work.WorkerParameters
import com.oloko.forecast.MainActivity
import com.oloko.forecast.R
import java.util.concurrent.TimeUnit

/**
 * Home-screen widget: today's (or a chosen day's) temperature and rain chart,
 * the current conditions, and ◀ day ▶ to browse the forecast. Data comes from
 * widget.json, written by the app after each fetch and by [RefreshWorker].
 * Between fetches, an alarm redraws it when the hour turns, so the chart's
 * "now" follows the clock, and [ReturnToTodayWorker] brings a picked day back
 * to today.
 */
class ForecastWidget : AppWidgetProvider() {
  override fun onUpdate(context: Context, manager: AppWidgetManager, ids: IntArray) {
    RefreshWorker.schedule(context, Settings.load(context).refreshMinutes)
    ids.forEach { render(context, manager, it) }
  }

  override fun onAppWidgetOptionsChanged(context: Context, manager: AppWidgetManager, id: Int, options: Bundle) {
    render(context, manager, id) // resized: redraw the chart at the new size
  }

  override fun onEnabled(context: Context) = RefreshWorker.schedule(context, Settings.load(context).refreshMinutes)

  override fun onDisabled(context: Context) {
    RefreshWorker.cancel(context)
    alarms(context).cancel(tick(context))
    WorkManager.getInstance(context).cancelUniqueWork(RETURN_WORK)
  }

  override fun onDeleted(context: Context, ids: IntArray) {
    prefs(context).edit().apply { ids.forEach { remove(dayKey(it)); remove(dayKey(it) + "_at") } }.apply()
  }

  override fun onReceive(context: Context, intent: Intent) {
    super.onReceive(context, intent)
    val id = intent.getIntExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
    when (intent.action) {
      ACTION_PREV, ACTION_NEXT -> {
        val forecast = Store.readForecast(context) ?: return
        val days = forecast.days.map { it.date }
        if (days.isEmpty()) return
        val current = selectedDate(context, id, forecast)
        val index = (days.indexOf(current) + if (intent.action == ACTION_NEXT) 1 else -1).coerceIn(0, days.size - 1)
        prefs(context).edit()
          .putString(dayKey(id), days[index])
          .putLong(dayKey(id) + "_at", System.currentTimeMillis())
          .apply()
        render(context, AppWidgetManager.getInstance(context), id)
        scheduleReturnToToday(context)
      }
      // Only redraws from the saved forecast: no network.
      ACTION_TICK, Intent.ACTION_TIME_CHANGED, Intent.ACTION_TIMEZONE_CHANGED -> updateAll(context)
      ACTION_REFRESH -> {
        prefs(context).edit().putBoolean(KEY_REFRESHING, true).apply()
        updateAll(context)
        RefreshWorker.refreshNow(context)
      }
    }
  }

  companion object {
    private const val ACTION_PREV = "com.oloko.forecast.widget.PREV"
    private const val ACTION_NEXT = "com.oloko.forecast.widget.NEXT"
    private const val ACTION_REFRESH = "com.oloko.forecast.widget.REFRESH"
    private const val ACTION_TICK = "com.oloko.forecast.widget.TICK"
    private const val PREFS = "forecast_widget"
    const val KEY_REFRESHING = "refreshing"
    private const val KEY_SCHEDULED_MINUTES = "scheduled_minutes"
    private const val RETURN_WORK = "forecast-widget-return-to-today"
    private const val HOUR_MS = 60 * 60 * 1000L

    private fun prefs(context: Context) = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
    private fun dayKey(id: Int) = "day_$id"

    /** Redraws every widget instance (after new data or settings). */
    fun updateAll(context: Context) {
      val manager = AppWidgetManager.getInstance(context)
      val ids = manager.getAppWidgetIds(ComponentName(context, ForecastWidget::class.java))
      if (ids.isEmpty()) return
      // The refresh interval may have changed in the app's settings.
      val minutes = Settings.load(context).refreshMinutes
      if (prefs(context).getInt(KEY_SCHEDULED_MINUTES, -1) != minutes) {
        RefreshWorker.schedule(context, minutes)
        prefs(context).edit().putInt(KEY_SCHEDULED_MINUTES, minutes).apply()
      }
      ids.forEach { render(context, manager, it) }
    }

    private fun alarms(context: Context) = context.getSystemService(AlarmManager::class.java)

    private fun tick(context: Context) = PendingIntent.getBroadcast(
      context, -1, Intent(context, ForecastWidget::class.java).setAction(ACTION_TICK),
      PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )

    /**
     * Sets the next redraw for when the hour turns at the location. A non-wakeup
     * alarm: with the screen off it waits until the phone wakes, which is when the
     * widget can be seen anyway. Inexact (no special permission): Android may run
     * it up to ~10 minutes late.
     */
    private fun scheduleTick(context: Context, forecast: Forecast?) {
      if (forecast == null) return
      val now = System.currentTimeMillis()
      val offset = forecast.utcOffsetSeconds * 1000L
      // +1 s so the redraw lands inside the new hour.
      val next = ((now + offset) / HOUR_MS + 1) * HOUR_MS - offset + 1000
      alarms(context).setWindow(AlarmManager.RTC, next, 60_000, tick(context))
    }

    /**
     * Redraws once the day just picked is due to go back to today. A one-off job
     * rather than an alarm: inexact alarms can run 10 minutes late, too coarse for
     * a 30 s to 20 min timer, while a job runs on time while the phone is in use.
     */
    private fun scheduleReturnToToday(context: Context) {
      val show = Settings.load(context).show
      if (!show.dayPicker || !show.returnToToday) return
      val request = OneTimeWorkRequestBuilder<ReturnToTodayWorker>()
        .setInitialDelay(show.returnAfterSeconds + 1L, TimeUnit.SECONDS)
        .build()
      WorkManager.getInstance(context).enqueueUniqueWork(RETURN_WORK, ExistingWorkPolicy.REPLACE, request)
    }

    /**
     * The day shown by widget [id]: the one picked with the arrows if still in the
     * forecast and, with "return to today" on, picked less than the set time ago;
     * else today. Without the arrows there is no way back, so always today.
     */
    private fun selectedDate(context: Context, id: Int, forecast: Forecast): String {
      val dates = forecast.days.map { it.date }
      val show = Settings.load(context).show
      val pickedAt = prefs(context).getLong(dayKey(id) + "_at", 0)
      val recent = !show.returnToToday || System.currentTimeMillis() - pickedAt < show.returnAfterSeconds * 1000L
      val saved = if (show.dayPicker && recent) prefs(context).getString(dayKey(id), null) else null
      val today = forecast.nowKey().take(10)
      return when {
        saved != null && saved in dates && saved >= today -> saved
        today in dates -> today
        else -> dates.firstOrNull() ?: today
      }
    }

    private fun render(context: Context, manager: AppWidgetManager, id: Int) {
      val s = Settings.load(context)
      val forecast = Store.readForecast(context)
      val show = s.show
      val layout = if (show.transparent) R.layout.widget_forecast_shadow else R.layout.widget_forecast
      val views = RemoteViews(context.packageName, layout)

      views.setInt(R.id.bg, "setColorFilter", s.card)
      views.setInt(R.id.bg, "setImageAlpha", show.background * 255 / 100)
      listOf(R.id.now_temp, R.id.day, R.id.clock_small, R.id.clock_big).forEach { views.setTextColor(it, s.fg) }
      listOf(R.id.location, R.id.summary, R.id.clock_date).forEach { views.setTextColor(it, s.muted) }

      // Clock: off, small (inline) or big (time + date). Tapping it opens the clock app.
      fun visible(on: Boolean) = if (on) View.VISIBLE else View.GONE
      views.setViewVisibility(R.id.clock_small, visible(show.clock == "small"))
      views.setViewVisibility(R.id.clock_big_block, visible(show.clock == "big"))
      val alarms = PendingIntent.getActivity(
        context, 1, (clockIntent(context) ?: Intent(context, MainActivity::class.java)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )
      views.setOnClickPendingIntent(R.id.clock_small, alarms)
      views.setOnClickPendingIntent(R.id.clock_big_block, alarms)
      listOf(R.id.prev, R.id.next, R.id.refresh).forEach { views.setTextColor(it, s.accent) }

      val open = PendingIntent.getActivity(
        context, 0, Intent(context, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
      )
      views.setOnClickPendingIntent(R.id.chart, open)
      views.setOnClickPendingIntent(R.id.info, open)
      views.setOnClickPendingIntent(R.id.prev, action(context, ACTION_PREV, id))
      views.setOnClickPendingIntent(R.id.next, action(context, ACTION_NEXT, id))
      views.setOnClickPendingIntent(R.id.refresh, action(context, ACTION_REFRESH, id))
      val refreshing = prefs(context).getBoolean(KEY_REFRESHING, false)
      views.setTextViewText(R.id.refresh, if (refreshing) "…" else "↻")

      if (forecast == null || forecast.days.isEmpty()) {
        views.setTextViewText(R.id.location, s.locationName ?: "Forecast")
        views.setTextViewText(R.id.now_temp, "")
        views.setTextViewText(R.id.now_icon, "")
        views.setTextViewText(R.id.day, "")
        views.setTextViewText(
          R.id.summary,
          s.text("Open Forecast once to show the weather here", "Abra o Forecast uma vez para mostrar o tempo aqui"),
        )
        views.setViewVisibility(R.id.chart, View.INVISIBLE)
        manager.updateAppWidget(id, views)
        return
      }

      // Parts the user switched off in the app's widget settings.
      // On narrow widgets the header has no room for the name (it ended up as "…"):
      // it moves to the start of the summary line instead.
      val widthDp = manager.getAppWidgetOptions(id).getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH)
      val narrow = widthDp in 1 until 300
      val locationInHeader = show.location && !narrow
      views.setViewVisibility(R.id.location, visible(locationInHeader))
      views.setViewVisibility(R.id.now_icon, visible(show.now))
      views.setViewVisibility(R.id.now_temp, visible(show.now))
      listOf(R.id.prev, R.id.day, R.id.next).forEach { views.setViewVisibility(it, visible(show.dayPicker)) }
      views.setViewVisibility(R.id.refresh, visible(show.refreshButton))
      val locationInSummary = show.location && narrow
      views.setViewVisibility(R.id.summary, visible(show.summary || locationInSummary))
      val headerShown = show.location || show.now || show.dayPicker || show.refreshButton || show.clock != "off"
      views.setViewVisibility(R.id.header, visible(headerShown))

      val nowKey = forecast.nowKey()
      val today = nowKey.take(10)
      val date = selectedDate(context, id, forecast)
      val dayIndex = forecast.days.indexOfFirst { it.date == date }
      val day = forecast.days[dayIndex]
      val hours = forecast.hours.filter { it.date == date }
      val now = forecast.hours.firstOrNull { it.hourKey == nowKey } ?: hours.firstOrNull()

      views.setTextViewText(R.id.location, forecast.locationName)
      views.setTextViewText(R.id.now_icon, now?.icon.orEmpty())
      views.setTextViewText(R.id.now_temp, now?.let { s.temp(it.temperature) }.orEmpty())
      views.setContentDescription(R.id.now_icon, now?.let { s.weather(it.description) })
      views.setTextViewText(R.id.day, s.dayLabel(date, today))
      views.setTextViewText(
        R.id.summary,
        if (narrow) {
          // Compact: "Rio de Janeiro · ↑28° ↓20° · 6.8 mm"
          listOfNotNull(
            forecast.locationName.takeIf { show.location && it.isNotEmpty() },
            "↑${s.temp(day.max)} ↓${s.temp(day.min)}".takeIf { show.summary },
            "${s.mm(day.precipitation)} mm".takeIf { show.summary },
          ).joinToString(" · ")
        } else {
          "↑ ${s.tempUnit(day.max)}  ↓ ${s.tempUnit(day.min)}  ·  💧 ${s.mm(day.precipitation)} mm" +
            (if (now != null && date == today) "  ·  ${s.weather(now.description)}" else "")
        },
      )
      // Dim an arrow at either end of the forecast.
      views.setTextColor(R.id.prev, if (dayIndex > 0) s.accent else s.line)
      views.setTextColor(R.id.next, if (dayIndex < forecast.days.size - 1) s.accent else s.line)

      // Draw the chart at the widget's size (header ~52 dp + padding).
      val density = context.resources.displayMetrics.density
      val options = manager.getAppWidgetOptions(id)
      val chartWidthDp = options.getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH).takeIf { it > 0 } ?: 250
      val heightDp = options.getInt(AppWidgetManager.OPTION_APPWIDGET_MAX_HEIGHT).takeIf { it > 0 } ?: 110
      val chartW = ((chartWidthDp - 20) * density).toInt()
      val chrome = 12 + (if (headerShown) 34 else 0) + (if (show.summary || locationInSummary) 16 else 0) +
        (if (show.clock == "big") 24 else 0)
      val chartH = ((heightDp - chrome).coerceAtLeast(48) * density).toInt()
      val nowIndex = if (date == today) hours.indexOfFirst { it.hourKey == nowKey } else -1
      views.setImageViewBitmap(R.id.chart, ChartRenderer.render(chartW, chartH, density, hours, nowIndex, s))
      views.setViewVisibility(R.id.chart, View.VISIBLE)
      manager.updateAppWidget(id, views)
      scheduleTick(context, forecast)
    }

    /** Phone makers' clock apps, tried in order (see <queries> in the manifest). */
    private val CLOCK_PACKAGES = listOf(
      "com.sec.android.app.clockpackage", // Samsung
      "com.google.android.deskclock", // Pixel and Google Clock
      "com.android.deskclock", // AOSP, Xiaomi, Huawei and others
      "com.oneplus.deskclock",
      "com.coloros.alarmclock", // Oppo / Realme
      "com.oplus.alarmclock",
      "com.motorola.timeweatherwidget",
      "com.asus.deskclock",
      "com.sonymobile.alarm",
    )

    /**
     * The phone's clock app. Opens it by package first: some clock apps (Samsung's)
     * don't answer the generic "show alarms" intent, which made the tap do nothing.
     */
    private fun clockIntent(context: Context): Intent? {
      val pm = context.packageManager
      CLOCK_PACKAGES.firstNotNullOfOrNull { pm.getLaunchIntentForPackage(it) }?.let { return it }
      val showAlarms = Intent(AlarmClock.ACTION_SHOW_ALARMS)
      return showAlarms.takeIf { it.resolveActivity(pm) != null }
    }

    private fun action(context: Context, action: String, id: Int): PendingIntent {
      val intent = Intent(context, ForecastWidget::class.java)
        .setAction(action)
        .putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id)
      // Distinct request code per widget and action, so each button keeps its own extras.
      val code = id * 4 + listOf(ACTION_PREV, ACTION_NEXT, ACTION_REFRESH).indexOf(action)
      return PendingIntent.getBroadcast(context, code, intent, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
    }
  }
}

/** Redraws the widgets from the saved forecast, so a picked day goes back to today. */
class ReturnToTodayWorker(context: Context, params: WorkerParameters) : Worker(context, params) {
  override fun doWork(): Result {
    ForecastWidget.updateAll(applicationContext)
    return Result.success()
  }
}
