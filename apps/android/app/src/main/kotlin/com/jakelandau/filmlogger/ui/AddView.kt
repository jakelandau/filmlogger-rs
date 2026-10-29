package com.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.Surface
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.key
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import uniffi.film_logger_ffi.Rating
import java.time.Instant
import kotlinx.coroutines.launch
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.Movie
import uniffi.film_logger_ffi.MovieSummary
import uniffi.film_logger_ffi.SearchPage
import uniffi.film_logger_ffi.Venue

/** The selected search/finder result the user is about to log. */
data class SelectedFilm(
    val id: Long,
    val title: String,
    val year: Int?,
    val posterPath: String?,
    val imdbId: String?,
) {
    companion object {
        fun fromSummary(r: MovieSummary): SelectedFilm =
            SelectedFilm(r.id, r.title, yearFrom(r.releaseDate), r.posterPath, null)

        fun fromMovie(m: Movie): SelectedFilm =
            SelectedFilm(m.id, m.title, yearFrom(m.releaseDate), m.posterPath, m.imdbId)
    }
}

/** A full TMDB poster URL, or null when the path is unknown. */
fun posterUrl(path: String?): String? = path?.takeIf { it.isNotBlank() }?.let {
    if (it.startsWith("http")) it else "https://image.tmdb.org/t/p/w342$it"
}

/**
 * The add flow: search TMDB (or find by IMDB id), pick a result, choose venue
 * and optional viewing format, and log the film.
 */
@Composable
fun AddView(
    logger: Logger,
    onSaved: () -> Unit,
    onError: (String) -> Unit,
) {
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var imdbQuery by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<SearchPage?>(null) }
    var selected by remember { mutableStateOf<SelectedFilm?>(null) }
    var busy by remember { mutableStateOf(false) }

    // detail form state (survives selection changes)
    var venue by remember { mutableStateOf(Venue.THEATRE) }
    var attachFormat by remember { mutableStateOf(false) }
    var format by remember { mutableStateOf<Format?>(null) }
    var watchedAtInput by remember { mutableStateOf(TimeUtils.toLocalInput(Instant.now().toString())) }
    var rating by remember { mutableStateOf<Rating?>(null) }
    var rewatch by remember { mutableStateOf(false) }
    var watchCount by remember { mutableStateOf("") }
    var notes by remember { mutableStateOf("") }

    fun doSearch() {
        if (query.isBlank()) return
        busy = true
        scope.launch {
            try {
                results = logger.search(query.trim(), 1)
            } catch (e: LoggerException) {
                onError(e.message ?: "search failed")
            } finally {
                busy = false
            }
        }
    }

    fun doFind() {
        if (imdbQuery.isBlank()) return
        busy = true
        scope.launch {
            try {
                selected = SelectedFilm.fromMovie(logger.findImdb(imdbQuery.trim()))
            } catch (e: LoggerException) {
                onError(e.message ?: "film not found")
            } finally {
                busy = false
            }
        }
    }

    fun doAdd() {
        val s = selected ?: return
        busy = true
        scope.launch {
            try {
                val f = if (attachFormat) format else null
                val log = logger.addFilm(s.id, venue, f)
                log.watchedAt = TimeUtils.fromLocalInput(watchedAtInput).ifEmpty { log.watchedAt }
                log.rating = rating
                log.rewatch = rewatch
                log.watchCount = watchCount.toIntOrNull()
                log.notes = notes.ifBlank { null }
                logger.update(log)
                onSaved()
            } catch (e: LoggerException) {
                onError(e.message ?: "add failed")
            } finally {
                busy = false
            }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(12.dp),
    ) {
        Row(Modifier.fillMaxWidth()) {
            OutlinedTextField(
                value = query,
                onValueChange = { query = it },
                modifier = Modifier.weight(1f),
                label = { Text("Film title") },
            )
            Spacer(Modifier.width(8.dp))
            Button(onClick = { doSearch() }, enabled = !busy) { Text("Search") }
        }
        Spacer(Modifier.height(8.dp))
        Row(Modifier.fillMaxWidth()) {
            OutlinedTextField(
                value = imdbQuery,
                onValueChange = { imdbQuery = it },
                modifier = Modifier.weight(1f),
                label = { Text("IMDB id (tt1234567)") },
            )
            Spacer(Modifier.width(8.dp))
            Button(onClick = { doFind() }, enabled = !busy) { Text("Find") }
        }
        Spacer(Modifier.height(12.dp))
        if (busy) {
            Box(
                modifier = Modifier.fillMaxWidth(),
                contentAlignment = Alignment.Center,
            ) {
                CircularProgressIndicator(Modifier.size(24.dp))
            }
            Spacer(Modifier.height(12.dp))
        }
        val s = selected
        if (s != null) {
            DetailCard(
                film = s,
                venue = venue,
                onVenue = { venue = it },
                attachFormat = attachFormat,
                onAttachFormat = { attachFormat = it },
                format = format,
                onFormat = { format = it },
                watchedAtInput = watchedAtInput,
                onWatchedAt = { watchedAtInput = it },
                rating = rating,
                onRating = { rating = it },
                rewatch = rewatch,
                onRewatch = { rewatch = it },
                watchCount = watchCount,
                onWatchCount = { watchCount = it },
                notes = notes,
                onNotes = { notes = it },
                busy = busy,
                onLog = { doAdd() },
                onBack = { selected = null },
            )
        } else {
            val page = results
            if (page != null) {
                Text("Results", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                ResultsList(page, busy = busy, onSelect = { r -> selected = SelectedFilm.fromSummary(r) })
            }
        }
    }
}

@Composable
private fun ResultsList(
    page: SearchPage,
    busy: Boolean,
    onSelect: (MovieSummary) -> Unit,
) {
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        items(page.results) { r ->
            val s = SelectedFilm.fromSummary(r)
            ResultRow(s, busy = busy, onClick = { onSelect(r) })
        }
    }
}

@Composable
private fun ResultRow(film: SelectedFilm, busy: Boolean, onClick: () -> Unit) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .clickable(onClick = onClick, enabled = !busy),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val poster = posterUrl(film.posterPath)
            if (poster != null) {
                AsyncImage(
                    model = poster,
                    contentDescription = null,
                    modifier = Modifier
                        .size(48.dp)
                        .clip(RoundedCornerShape(4.dp)),
                    contentScale = ContentScale.Crop,
                )
            }
            Spacer(Modifier.width(12.dp))
            Column(Modifier.weight(1f)) {
                Text(film.title, style = MaterialTheme.typography.titleMedium)
                Text(
                    "${film.year ?: ""}".trim().let { if (it.isEmpty()) "" else " · $it" },
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}

@Composable
private fun DetailCard(
    film: SelectedFilm,
    venue: Venue,
    onVenue: (Venue) -> Unit,
    attachFormat: Boolean,
    onAttachFormat: (Boolean) -> Unit,
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
    busy: Boolean,
    onLog: () -> Unit,
    onBack: () -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.medium,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
        ) {
            Row {
                val poster = posterUrl(film.posterPath)
                if (poster != null) {
                    AsyncImage(
                        model = poster,
                        contentDescription = null,
                        modifier = Modifier
                            .size(96.dp)
                            .clip(RoundedCornerShape(8.dp)),
                        contentScale = ContentScale.Crop,
                    )
                    Spacer(Modifier.width(12.dp))
                }
                Column(Modifier.weight(1f)) {
                    Text(
                        if (film.year != null) "${film.title} (${film.year})" else film.title,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    if (film.imdbId != null) {
                        Text(
                            film.imdbId,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Text(
                        FormatText.summary(format),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            Spacer(Modifier.height(12.dp))
            TextButton(onClick = onBack) { Text("‹ Back to results") }
            Spacer(Modifier.height(8.dp))

            // venue
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                RadioButton(
                    selected = venue == Venue.THEATRE,
                    onClick = { onVenue(Venue.THEATRE) },
                )
                Text("Theatre")
                Spacer(Modifier.width(16.dp))
                RadioButton(
                    selected = venue == Venue.HOME,
                    onClick = { onVenue(Venue.HOME) },
                )
                Text("Home")
            }
            Spacer(Modifier.height(8.dp))

            // format attach
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                Checkbox(
                    checked = attachFormat,
                    onCheckedChange = {
                        onAttachFormat(it)
                        if (it && format == null) onFormat(FormatText.build(FormatParts.forVenue(venue)))
                    },
                )
                Text("Attach viewing format")
            }
            if (attachFormat) {
                key(venue) {
                    FormatPicker(venue, format, onFormat)
                }
            }
            Spacer(Modifier.height(8.dp))

            // watched at
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

            // rating
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

            // rewatch + watch count
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

            // notes
            OutlinedTextField(
                value = notes,
                onValueChange = onNotes,
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Notes") },
            )
            Spacer(Modifier.height(12.dp))

            Button(onClick = onLog, enabled = !busy, modifier = Modifier.fillMaxWidth()) {
                Text("Log this film")
            }
        }
    }
}
