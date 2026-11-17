package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

/** A prominent IMAX / DCI / FILM badge shown before the format summary. */
@Composable
fun FormatFamilyBadge(format: uniffi.filmlogger_rs_ffi.Format?) {
    val family = formatFamily(format) ?: return
    val background = when (family) {
        "IMAX" -> Color(0xFF243642)
        "DCI" -> Color(0xFF1F3B31)
        "FILM" -> Color(0xFF4A2A16)
        else -> Color(0xFF35254A)
    }
    val foreground = when (family) {
        "IMAX" -> Color(0xFF8FC4E8)
        "DCI" -> Color(0xFF8FE3C0)
        "FILM" -> Color(0xFFFFB27A)
        else -> Color(0xFFD9B8FF)
    }
    Text(
        family,
        modifier = Modifier
            .background(background, RoundedCornerShape(4.dp))
            .padding(horizontal = 6.dp, vertical = 1.dp),
        style = MaterialTheme.typography.labelSmall,
        color = foreground,
        fontWeight = FontWeight.Bold,
    )
}
