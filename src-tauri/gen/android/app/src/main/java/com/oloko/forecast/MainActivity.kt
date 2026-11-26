package com.oloko.forecast

import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.graphics.Insets
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.oloko.forecast.widget.ForecastWidget

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    resizeForKeyboard()
  }

  /** Leaving the app: show its latest forecast and settings on the widget. */
  override fun onPause() {
    super.onPause()
    ForecastWidget.updateAll(this)
  }

  /**
   * Edge-to-edge windows are not resized for the keyboard (adjustResize is ignored
   * since Android 15), so the keyboard would cover the bottom of the WebView. Pad
   * the content by the keyboard height instead: the WebView shrinks, and bottom
   * sheets and inputs stay above the keyboard.
   */
  private fun resizeForKeyboard() {
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
      view.setPadding(0, 0, 0, ime.bottom)
      if (ime.bottom == 0) return@setOnApplyWindowInsetsListener insets

      // The keyboard now covers the navigation bar: tell the page there is no
      // bottom inset left, so it doesn't add a gap above the keyboard.
      fun noBottom(type: Int): Insets =
        insets.getInsets(type).let { Insets.of(it.left, it.top, it.right, 0) }
      WindowInsetsCompat.Builder(insets)
        .setInsets(WindowInsetsCompat.Type.ime(), Insets.NONE)
        .setInsets(WindowInsetsCompat.Type.navigationBars(), noBottom(WindowInsetsCompat.Type.navigationBars()))
        .setInsets(WindowInsetsCompat.Type.systemBars(), noBottom(WindowInsetsCompat.Type.systemBars()))
        .build()
    }
  }
}
