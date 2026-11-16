package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
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
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import java.time.Instant
import java.time.ZoneOffset
import uniffi.film_logger_ffi.Rating
import uniffi.film_logger_ffi.Venue
import uniffi.film_logger_ffi.WatchLog

private enum class LogSort { RECENT, OLDEST, TITLE }

// Density-independent spacing keeps the card rhythm stable across devices.
private val CardPadding = 16.dp
private val PosterGap = 16.dp
private val RowGap = 6.dp
private val InlineGap = 6.dp
private val ActionGap = 12.dp

private fun utcMillis(log: WatchLog): Long = Instant.parse(log.watchedAt).toEpochMilli()

private fun utcYear(log: WatchLog): Int =
    Instant.parse(log.watchedAt).atZone(ZoneOffset.UTC).year

/**
 * Filter and sort logs for display, matching the desktop list semantics:
 * text matches the title or notes case-insensitively, the year is the UTC
 * year, and sort ties fall back to recency.
 */
private fun filterLogs(
    logs: List<WatchLog>,
    text: String,
    venue: Venue?,
    year: Int?,
    rating: Rating?,
    sort: LogSort,
): List<WatchLog> {
    val needle = text.trim().lowercase()
    val matched = logs.filter { log ->
        (venue == null || log.venue == venue) &&
            (year == null || utcYear(log) == year) &&
            (rating == null || log.rating == rating) &&
            (needle.isEmpty() || log.title.lowercase().contains(needle))
    }
    val recent: Comparator<WatchLog> =
        compareByDescending<WatchLog> { utcMillis(it) }.thenByDescending { it.id }
    return when (sort) {
        LogSort.RECENT -> matched.sortedWith(recent)
        LogSort.OLDEST -> matched.sortedWith(recent.reversed())
        LogSort.TITLE -> matched.sortedWith(
            compareBy<WatchLog> { it.title.lowercase() }.then(recent),
        )
    }
}

/** The log list: every recorded viewing, filterable and sortable. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun LogsView(
    logs: List<WatchLog>,
    selectedBoard: String,
    onEdit: (Long) -> Unit,
    onAdd: () -> Unit,
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

    var text by remember { mutableStateOf("") }
    var venue by remember { mutableStateOf<Venue?>(null) }
    var year by remember { mutableStateOf<Int?>(null) }
    var rating by remember { mutableStateOf<Rating?>(null) }
    var sort by remember { mutableStateOf(LogSort.RECENT) }

    val years = logs.map { utcYear(it) }.distinct().sortedDescending()
    val visible = filterLogs(logs, text, venue, year, rating, sort)

    Column(Modifier.fillMaxSize()) {
        Column(Modifier.padding(horizontal = 16.dp, vertical = 12.dp)) {
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Search logs") },
                singleLine = true,
            )
            Spacer(Modifier.height(10.dp))
            FlowRow(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                FilterChip(selected = venue == null, onClick = { venue = null }, label = { Text("All venues") })
                FilterChip(selected = venue == Venue.THEATRE, onClick = { venue = Venue.THEATRE }, label = { Text("Theatre") })
                FilterChip(selected = venue == Venue.HOME, onClick = { venue = Venue.HOME }, label = { Text("Home") })
                FilterChip(selected = rating == null, onClick = { rating = null }, label = { Text("Any rating") })
                FilterChip(selected = rating == Rating.UP, onClick = { rating = Rating.UP }, label = { Text("👍 Up") })
                FilterChip(selected = rating == Rating.DOWN, onClick = { rating = Rating.DOWN }, label = { Text("👎 Down") })
            }
            Spacer(Modifier.height(6.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                OptionField(
                    "Year",
                    year?.toString() ?: "all",
                    listOf("all" to "All years") + years.map { it.toString() to it.toString() },
                    Modifier.weight(1f),
                ) { key -> year = key.toIntOrNull() }
                OptionField(
                    "Sort",
                    when (sort) {
                        LogSort.RECENT -> "recent"
                        LogSort.OLDEST -> "oldest"
                        LogSort.TITLE -> "title"
                    },
                    listOf("recent" to "Most recent", "oldest" to "Oldest", "title" to "Title"),
                    Modifier.weight(1f),
                ) { key ->
                    sort = when (key) {
                        "oldest" -> LogSort.OLDEST
                        "title" -> LogSort.TITLE
                        else -> LogSort.RECENT
                    }
                }
            }
        }

        if (visible.isEmpty()) {
            Text(
                "No viewings match these filters.",
                modifier = Modifier.padding(16.dp),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            return@Column
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            items(visible) { log ->
                LogRow(
                    log,
                    selectedBoard,
                    onEdit = { onEdit(log.id) },
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun LogRow(
    log: WatchLog,
    selectedBoard: String,
    onEdit: () -> Unit,
) {
    val year = log.releaseYear
    val title = if (year != null) "${log.title} ($year)" else log.title
    val cert = certificationFor(log, selectedBoard)
    val zone = log.timezone?.let { TimeUtils.zone(it) } ?: TimeUtils.systemZone()
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.large,
    ) {
        Column(Modifier.fillMaxWidth().padding(CardPadding)) {
            Row(verticalAlignment = Alignment.Top) {
                val poster = posterUrl(log.posterPath)
                if (poster != null) {
                    AsyncImage(
                        model = poster,
                        contentDescription = null,
                        modifier = Modifier
                            .width(56.dp)
                            .height(84.dp)
                            .clip(RoundedCornerShape(6.dp)),
                    )
                } else {
                    Box(
                        modifier = Modifier
                            .width(56.dp)
                            .height(84.dp)
                            .clip(RoundedCornerShape(6.dp))
                            .background(MaterialTheme.colorScheme.surface),
                    )
                }
                Spacer(Modifier.width(PosterGap))
                Column(Modifier.weight(1f)) {
                    // Row 1: identity and venue.
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(InlineGap),
                        verticalArrangement = Arrangement.spacedBy(2.dp),
                    ) {
                        Text(
                            title,
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        if (log.rewatch) {
                            Text(
                                "Rewatch",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.primary,
                            )
                        }
                    }
                    Spacer(Modifier.height(2.dp))
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(InlineGap),
                        verticalArrangement = Arrangement.spacedBy(2.dp),
                    ) {
                        Text(
                            TimeUtils.display(log.watchedAt, zone),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Text(
                            venueText(log.venue),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }

                    // Row 2: format information.
                    Spacer(Modifier.height(RowGap))
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(InlineGap),
                        verticalArrangement = Arrangement.spacedBy(2.dp),
                    ) {
                        FormatFamilyBadge(log.format)
                        Text(
                            log.formatSummary.ifEmpty { "—" },
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                    }
                    if (formatFlags(log.format).isNotEmpty()) {
                        Spacer(Modifier.height(4.dp))
                        FormatFlagsRow(log.format)
                    }

                    // Row 3: content rating and reaction.
                    Spacer(Modifier.height(RowGap))
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        if (cert != null) {
                            CertificationIcon(selectedBoard, cert)
                            Spacer(Modifier.width(4.dp))
                        }
                        Text(
                            cert ?: "Unrated",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        log.rating?.let { rating ->
                            Spacer(Modifier.width(12.dp))
                            Text(
                                ratingText(rating),
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurface,
                            )
                        }
                    }
                }
            }
            Spacer(Modifier.height(ActionGap))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TextButton(onClick = onEdit) { Text("Edit", style = MaterialTheme.typography.labelLarge) }
            }
        }
    }
}

/** The film's certification for the user's selected rating board, or null
 *  when unknown (the film has no certifications for that board). */
private fun certificationFor(log: WatchLog, board: String): String? =
    log.certifications?.values?.firstOrNull { it.region == board }?.rating

private fun venueText(venue: Venue): String = when (venue) {
    Venue.THEATRE -> "🎭 Theatre"
    Venue.HOME -> "🏠 Home"
}

private fun ratingText(rating: Rating): String = when (rating) {
    Rating.UP -> "👍 Liked"
    Rating.DOWN -> "👎 Disliked"
}
