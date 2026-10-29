package com.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import uniffi.film_logger_ffi.Rating
import uniffi.film_logger_ffi.RatingBoard
import uniffi.film_logger_ffi.Venue
import uniffi.film_logger_ffi.WatchLog

/** The log list: every recorded viewing, most recent first. */
@Composable
fun LogsView(
    logs: List<WatchLog>,
    selectedBoard: RatingBoard,
    onEdit: (Long) -> Unit,
    onAdd: () -> Unit,
    onDelete: (Long) -> Unit,
) {
    if (logs.isEmpty()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(24.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text("No films logged yet", style = MaterialTheme.typography.titleMedium)
            Spacer(Modifier.height(12.dp))
            TextButton(onClick = onAdd) { Text("Add your first film") }
        }
        return
    }
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(12.dp),
    ) {
        items(logs) { log ->
            LogRow(
                log,
                selectedBoard,
                onEdit = { onEdit(log.id) },
                onDelete = { onDelete(log.id) },
            )
        }
    }
}

@Composable
private fun LogRow(
    log: WatchLog,
    selectedBoard: RatingBoard,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    val year = log.releaseYear
    val title = if (year != null) "${log.title} ($year)" else log.title
    val cert = certificationFor(log, selectedBoard)
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.medium,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val poster = posterUrl(log.posterPath)
            if (poster != null) {
                AsyncImage(
                    model = poster,
                    contentDescription = null,
                    modifier = Modifier
                        .width(40.dp)
                        .height(60.dp)
                        .clip(RoundedCornerShape(4.dp)),
                )
            } else {
                Box(
                    modifier = Modifier
                        .width(40.dp)
                        .height(60.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.surface),
                )
            }
            Spacer(Modifier.width(12.dp))
            Column(Modifier.weight(1f)) {
                Text(
                    title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(Modifier.height(2.dp))
                Text(
                    "${TimeUtils.display(log.watchedAt)} · ${venueText(log.venue)} · ${cert ?: "unrated"}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                val detail = buildString {
                    if (log.format != null) append("  ·  ").append(FormatText.summary(log.format))
                    log.rating?.let { rating -> append("  ·  ").append(ratingText(rating)) }
                    if (log.rewatch) append("  ·  rewatch")
                }
                if (detail.isNotEmpty()) {
                    Spacer(Modifier.height(2.dp))
                    Text(detail, style = MaterialTheme.typography.bodySmall)
                }
            }
            TextButton(onClick = onEdit) { Text("Edit") }
            TextButton(onClick = onDelete) { Text("×") }
        }
    }
}

/** The film's certification for the user's selected rating board, or null
 *  when unknown (the film has no certifications for that board). */
private fun certificationFor(log: WatchLog, board: RatingBoard): String? =
    log.certifications?.let { c ->
        when (board) {
            RatingBoard.MPA -> c.us
            RatingBoard.BBFC -> c.gb
            RatingBoard.FSK -> c.de
        }
    }

private fun venueText(venue: Venue): String = when (venue) {
    Venue.THEATRE -> "🎭 Theatre"
    Venue.HOME -> "🏠 Home"
}

private fun ratingText(rating: Rating): String = when (rating) {
    Rating.UP -> "👍"
    Rating.DOWN -> "👎"
}
