package com.oloko.forecast.widget

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.DashPathEffect
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import kotlin.math.max
import kotlin.math.min

/** Draws the day's temperature line and rain bars, like the app's HourChart. */
object ChartRenderer {
  fun render(widthPx: Int, heightPx: Int, density: Float, hours: List<Hour>, nowIndex: Int, s: Settings): Bitmap {
    val w = widthPx.coerceAtLeast(1)
    val h = heightPx.coerceAtLeast(1)
    val bitmap = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
    val canvas = Canvas(bitmap)
    val dp = { v: Float -> v * density }
    val n = hours.size
    if (n < 2) return bitmap

    val padX = dp(8f)
    val labelH = dp(14f)
    val tempTop = dp(16f)
    val tempBottom = (h - labelH) * 0.62f
    val rainTop = (h - labelH) * 0.70f
    val rainBottom = h - labelH - dp(2f)
    val step = (w - 2 * padX) / (n - 1)
    fun x(i: Int) = padX + i * step

    // Rain bars, scaled to the day's peak (at least 1 mm/h). Hidden: the line takes the space.
    val tempBottomFinal = if (s.show.rain) tempBottom else rainBottom - dp(6f)
    if (!s.show.rain) {
      renderLine(canvas, w, h, dp, hours, nowIndex, s, padX, labelH, tempTop, tempBottomFinal, rainBottom)
      return bitmap
    }
    val rainMax = max(1.0, hours.maxOf { it.precipitation })
    val barW = max(dp(2f), step * 0.6f)
    val rainPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = s.rain }
    hours.forEachIndexed { i, hour ->
      if (hour.precipitation > 0.001) {
        val bh = max(dp(1.5f), ((hour.precipitation / rainMax) * (rainBottom - rainTop)).toFloat())
        rainPaint.alpha = if (i == nowIndex) 255 else 150
        canvas.drawRoundRect(RectF(x(i) - barW / 2, rainBottom - bh, x(i) + barW / 2, rainBottom), dp(1.5f), dp(1.5f), rainPaint)
      }
    }
    renderLine(canvas, w, h, dp, hours, nowIndex, s, padX, labelH, tempTop, tempBottom, rainBottom)
    return bitmap
  }

  /** Baseline, temperature curve, hour labels, high/low and the "now" marker. */
  private fun renderLine(
    canvas: Canvas, w: Int, h: Int, dp: (Float) -> Float, hours: List<Hour>, nowIndex: Int, s: Settings,
    padX: Float, labelH: Float, tempTop: Float, tempBottom: Float, rainBottom: Float,
  ) {
    val n = hours.size
    val step = (w - 2 * padX) / (n - 1)
    fun x(i: Int) = padX + i * step
    val temps = hours.map { s.toUnit(it.temperature) }
    val tMin = temps.min()
    val tMax = temps.max()
    val span = max(tMax - tMin, 2.0)
    val mid = (tMax + tMin) / 2
    fun y(t: Double) = (tempBottom - ((t - (mid - span / 2)) / span) * (tempBottom - tempTop)).toFloat()
    canvas.drawLine(padX, rainBottom, w - padX, rainBottom, Paint().apply { color = s.line; strokeWidth = dp(1f) })

    // Smooth temperature curve (Catmull-Rom → cubic Bézier), with a fading fill.
    val pts = temps.mapIndexed { i, t -> x(i) to y(t) }
    val line = Path().apply {
      moveTo(pts[0].first, pts[0].second)
      for (i in 0 until n - 1) {
        val (x0, y0) = pts[max(0, i - 1)]
        val (x1, y1) = pts[i]
        val (x2, y2) = pts[i + 1]
        val (x3, y3) = pts[min(n - 1, i + 2)]
        cubicTo(x1 + (x2 - x0) / 6, y1 + (y2 - y0) / 6, x2 - (x3 - x1) / 6, y2 - (y3 - y1) / 6, x2, y2)
      }
    }
    val fill = Path(line).apply {
      lineTo(x(n - 1), tempBottom + dp(6f))
      lineTo(x(0), tempBottom + dp(6f))
      close()
    }
    val fade = (s.temp and 0x00ffffff)
    canvas.drawPath(fill, Paint(Paint.ANTI_ALIAS_FLAG).apply {
      shader = LinearGradient(0f, tempTop, 0f, tempBottom + dp(6f), fade or (0x55 shl 24), fade, Shader.TileMode.CLAMP)
    })
    canvas.drawPath(line, Paint(Paint.ANTI_ALIAS_FLAG).apply {
      color = s.temp
      style = Paint.Style.STROKE
      strokeWidth = dp(2.2f)
      strokeJoin = Paint.Join.ROUND
      strokeCap = Paint.Cap.ROUND
    })

    val text = Paint(Paint.ANTI_ALIAS_FLAG).apply { textSize = dp(10f); color = s.muted; textAlign = Paint.Align.CENTER }

    // Hour labels every 6 h.
    hours.forEachIndexed { i, hour ->
      val hh = hour.time.substring(11, 13).toIntOrNull() ?: return@forEachIndexed
      if (hh % 6 == 0) canvas.drawText("${hh.toString().padStart(2, '0')}h", x(i).coerceIn(dp(12f), w - dp(12f)), h - dp(2f), text)
    }

    // Day's high and low, at their points (skipped when "now" sits there).
    val hi = temps.indexOf(tMax)
    val lo = temps.indexOf(tMin)
    text.color = s.muted
    if (hi != nowIndex) canvas.drawText("${Math.round(tMax)}°", x(hi).coerceIn(dp(12f), w - dp(12f)), y(tMax) - dp(5f), text)
    if (lo != nowIndex && lo != hi) canvas.drawText("${Math.round(tMin)}°", x(lo).coerceIn(dp(12f), w - dp(12f)), y(tMin) + dp(13f), text)

    // "Now" marker.
    if (nowIndex in 0 until n) {
      val nx = x(nowIndex)
      val ny = y(temps[nowIndex])
      canvas.drawLine(nx, tempTop - dp(4f), nx, rainBottom, Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = s.fg; alpha = 70; strokeWidth = dp(1f); pathEffect = DashPathEffect(floatArrayOf(dp(3f), dp(3f)), 0f)
      })
      canvas.drawCircle(nx, ny, dp(4.5f), Paint(Paint.ANTI_ALIAS_FLAG).apply { color = s.card })
      canvas.drawCircle(nx, ny, dp(4.5f), Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = s.temp; style = Paint.Style.STROKE; strokeWidth = dp(2.2f)
      })
      text.color = s.fg
      text.typeface = Typeface.DEFAULT_BOLD
      text.textSize = dp(11f)
      canvas.drawText("${Math.round(temps[nowIndex])}°", nx.coerceIn(dp(14f), w - dp(14f)), max(dp(11f), ny - dp(9f)), text)
    }
  }
}
