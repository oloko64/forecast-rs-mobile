package com.oloko.forecast.widget

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.view.View
import android.widget.RemoteViews
import com.oloko.forecast.MainActivity
import com.oloko.forecast.R

/**
 * Home-screen widget: today's (or a chosen day's) temperature and rain chart,
 * the current conditions, and ◀ day ▶ to browse the forecast. Data comes from
 * widget.json, written by the app after each fetch and by [RefreshWorker].
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

  override fun onDisabled(context: Context) = RefreshWorker.cancel(context)

  override fun onDeleted(context: Context, ids: IntArray) {
    prefs(context).edit().apply { ids.forEach { remove(dayKey(it)) } }.apply()
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
        prefs(context).edit().putString(dayKey(id), days[index]).apply()
        render(context, AppWidgetManager.getInstance(context), id)
      }
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
    private const val PREFS = "forecast_widget"
    const val KEY_REFRESHING = "refreshing"
    private const val KEY_SCHEDULED_MINUTES = "scheduled_minutes"

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

    /**
     * The day shown by widget [id]: the one picked with the arrows if still in the
     * forecast, else today. Without the arrows there is no way back, so always today.
     */
    private fun selectedDate(context: Context, id: Int, forecast: Forecast): String {
      val dates = forecast.days.map { it.date }
      val arrows = Settings.load(context).show.dayPicker
      val saved = if (arrows) prefs(context).getString(dayKey(id), null) else null
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
      val views = RemoteViews(context.packageName, R.layout.widget_forecast)

      views.setInt(R.id.bg, "setColorFilter", s.card)
      listOf(R.id.now_temp, R.id.day).forEach { views.setTextColor(it, s.fg) }
      listOf(R.id.location, R.id.summary).forEach { views.setTextColor(it, s.muted) }
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
      val show = s.show
      fun visible(on: Boolean) = if (on) View.VISIBLE else View.GONE
      views.setViewVisibility(R.id.location, visible(show.location))
      views.setViewVisibility(R.id.now_icon, visible(show.now))
      views.setViewVisibility(R.id.now_temp, visible(show.now))
      listOf(R.id.prev, R.id.day, R.id.next).forEach { views.setViewVisibility(it, visible(show.dayPicker)) }
      views.setViewVisibility(R.id.refresh, visible(show.refreshButton))
      views.setViewVisibility(R.id.summary, visible(show.summary))
      val headerShown = show.location || show.now || show.dayPicker || show.refreshButton
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
        "↑ ${s.tempUnit(day.max)}  ↓ ${s.tempUnit(day.min)}  ·  💧 ${s.mm(day.precipitation)} mm" +
          (if (now != null && date == today) "  ·  ${s.weather(now.description)}" else ""),
      )
      // Dim an arrow at either end of the forecast.
      views.setTextColor(R.id.prev, if (dayIndex > 0) s.accent else s.line)
      views.setTextColor(R.id.next, if (dayIndex < forecast.days.size - 1) s.accent else s.line)

      // Draw the chart at the widget's size (header ~52 dp + padding).
      val density = context.resources.displayMetrics.density
      val options = manager.getAppWidgetOptions(id)
      val widthDp = options.getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH).takeIf { it > 0 } ?: 250
      val heightDp = options.getInt(AppWidgetManager.OPTION_APPWIDGET_MAX_HEIGHT).takeIf { it > 0 } ?: 110
      val chartW = ((widthDp - 20) * density).toInt()
      val chrome = 12 + (if (headerShown) 34 else 0) + (if (show.summary) 16 else 0)
      val chartH = ((heightDp - chrome).coerceAtLeast(48) * density).toInt()
      val nowIndex = if (date == today) hours.indexOfFirst { it.hourKey == nowKey } else -1
      views.setImageViewBitmap(R.id.chart, ChartRenderer.render(chartW, chartH, density, hours, nowIndex, s))
      views.setViewVisibility(R.id.chart, View.VISIBLE)
      manager.updateAppWidget(id, views)
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
