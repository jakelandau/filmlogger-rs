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
fun FormatFamilyBadge(format: uniffi.film_logger_ffi.Format?) {
    val family = formatFamily(format) ?: return
    val background = when (family) {
        "IMAX" -> Color(0xFF4A3B1F)
        "DCI" -> Color(0xFF243642)
        "FILM" -> Color(0xFF3B2A25)
        else -> MaterialTheme.colorScheme.surface
    }
    val foreground = when (family) {
        "IMAX" -> Color(0xFFE7C873)
        "DCI" -> Color(0xFF8FC4E8)
        "FILM" -> Color(0xFFD9A48A)
        else -> MaterialTheme.colorScheme.onSurfaceVariant
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
