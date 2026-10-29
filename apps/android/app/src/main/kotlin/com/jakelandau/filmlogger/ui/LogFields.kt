package com.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import java.time.Instant
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.Rating
import uniffi.film_logger_ffi.Venue

/**
 * The venue, viewing-format, watched-at, rating, rewatch, watch-count, and
 * notes controls shared by the add and edit views.
 */
@Composable
fun LogFields(
    venue: Venue,
    onVenue: (Venue) -> Unit,
    format: Format?,
    onFormat: (Format) -> Unit,
    watchedAtInput: String,
    onWatchedAt: (String) -> Unit,
    rating: Rating?,
    onRating: (Rating?) -> Unit,
    rewatch: Boolean,
    onRewatch: (Boolean) -> Unit,
    watchCount: String,
    onWatchCount: (String) -> Unit,
    notes: String,
    onNotes: (String) -> Unit,
) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        RadioButton(selected = venue == Venue.THEATRE, onClick = { onVenue(Venue.THEATRE) })
        Text("Theatre")
        Spacer(Modifier.width(16.dp))
        RadioButton(selected = venue == Venue.HOME, onClick = { onVenue(Venue.HOME) })
        Text("Home")
    }
    Spacer(Modifier.height(8.dp))

    key(venue) {
        FormatPicker(venue, format, onFormat)
    }
    Spacer(Modifier.height(8.dp))

    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        OutlinedTextField(
            value = watchedAtInput,
            onValueChange = onWatchedAt,
            modifier = Modifier.weight(1f),
            label = { Text("Watched (local: yyyy-MM-dd HH:mm)") },
        )
        Spacer(Modifier.width(8.dp))
        OutlinedButton(onClick = { onWatchedAt(TimeUtils.toLocalInput(Instant.now().toString())) }) {
            Text("Now")
        }
    }
    Spacer(Modifier.height(8.dp))

    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Text("Rating:")
        Spacer(Modifier.width(8.dp))
        RadioButton(selected = rating == null, onClick = { onRating(null) })
        Text("none")
        Spacer(Modifier.width(8.dp))
        RadioButton(selected = rating == Rating.UP, onClick = { onRating(Rating.UP) })
        Text("👍")
        Spacer(Modifier.width(8.dp))
        RadioButton(selected = rating == Rating.DOWN, onClick = { onRating(Rating.DOWN) })
        Text("👎")
    }
    Spacer(Modifier.height(8.dp))

    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Checkbox(checked = rewatch, onCheckedChange = onRewatch)
        Text("Rewatch")
        Spacer(Modifier.width(16.dp))
        Text("Count:")
        Spacer(Modifier.width(8.dp))
        OutlinedTextField(
            value = watchCount,
            onValueChange = onWatchCount,
            modifier = Modifier.width(80.dp),
            label = { Text("#") },
        )
    }
    Spacer(Modifier.height(8.dp))

    OutlinedTextField(
        value = notes,
        onValueChange = onNotes,
        modifier = Modifier.fillMaxWidth(),
        label = { Text("Notes") },
    )
}
