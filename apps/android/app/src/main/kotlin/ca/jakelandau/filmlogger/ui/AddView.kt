package ca.jakelandau.filmlogger.ui

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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import java.time.Instant
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.film_logger_ffi.FilmId
import uniffi.film_logger_ffi.FilmMetadata
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.FormatCatalog
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.Movie
import uniffi.film_logger_ffi.MovieSummary
import uniffi.film_logger_ffi.NewLogDetails
import uniffi.film_logger_ffi.Rating
import uniffi.film_logger_ffi.SearchPage
import uniffi.film_logger_ffi.Venue

/** The selected search/finder result the user is about to log. */
data class SelectedFilm(
    val id: Long,
    val title: String,
    val year: Int?,
    val posterPath: String?,
    val imdbId: String?,
    /** Set when logging another viewing of an already-stored film. */
    val filmId: FilmId? = null,
) {
    companion object {
        fun fromSummary(r: MovieSummary): SelectedFilm =
            SelectedFilm(r.id, r.title, yearFrom(r.releaseDate), r.posterPath, null)

        fun fromMovie(m: Movie): SelectedFilm =
            SelectedFilm(m.id, m.title, yearFrom(m.releaseDate), m.posterPath, m.imdbId)

        fun fromStored(f: FilmMetadata): SelectedFilm =
            SelectedFilm(f.tmdbId ?: 0L, f.title, f.releaseYear, f.posterPath, f.imdbId, f.filmId)
    }
}

private enum class AddStage { CONFIRM, FORMAT, DETAILS }

/** A full TMDB poster URL, or null when the path is unknown. */
fun posterUrl(path: String?): String? = path?.takeIf { it.isNotBlank() }?.let {
    if (it.startsWith("http")) it else "https://image.tmdb.org/t/p/w342$it"
}

/**
 * The add flow: search TMDB (or pick a popular title, or find by IMDB id),
 * confirm the film, choose venue and viewing format with the viewing
 * details, and log it. With [preselected], it logs another viewing of an
 * already-stored film in one step.
 */
@Composable
fun AddView(
    logger: Logger,
    catalog: FormatCatalog,
    preselected: FilmMetadata?,
    onSaved: () -> Unit,
    onError: (String) -> Unit,
) {
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var imdbQuery by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<SearchPage?>(null) }
    var popular by remember { mutableStateOf<SearchPage?>(null) }
    var selected by remember { mutableStateOf(preselected?.let { SelectedFilm.fromStored(it) }) }
    var stage by remember {
        mutableStateOf(if (preselected != null) AddStage.FORMAT else AddStage.CONFIRM)
    }
    var busy by remember { mutableStateOf(false) }

    // detail form state (survives selection changes)
    var venue by remember { mutableStateOf(Venue.THEATRE) }
    var format by remember { mutableStateOf<Format?>(null) }
    var timezone by remember { mutableStateOf(TimeUtils.systemZone().id) }
    var watchedAtInput by remember {
        mutableStateOf(
            TimeUtils.toLocalInput(Instant.now().toString(), TimeUtils.zone(timezone)),
        )
    }
    var rating by remember { mutableStateOf<Rating?>(null) }

    LaunchedEffect(Unit) {
        if (preselected == null) {
            try {
                popular = withContext(Dispatchers.IO) { logger.popularMovies(1) }
            } catch (e: LoggerException) {
                onError(e.message ?: "popular films failed")
            }
        }
    }

    fun doSearch() {
        if (query.isBlank()) return
        busy = true
        scope.launch {
            try {
                results = withContext(Dispatchers.IO) { logger.search(query.trim(), 1) }
            } catch (e: LoggerException) {
                onError(e.message ?: "search failed")
            } finally {
                busy = false
            }
        }
    }

    fun loadMore() {
        val page = results ?: return
        if (page.page >= page.totalPages) return
        busy = true
        scope.launch {
            try {
                val next = withContext(Dispatchers.IO) { logger.search(query.trim(), page.page + 1) }
                results = SearchPage(
                    next.page,
                    page.results + next.results,
                    next.totalPages,
                    next.totalResults,
                )
            } catch (e: LoggerException) {
                onError(e.message ?: "search failed")
            } finally {
                busy = false
            }
        }
    }

    fun loadMorePopular() {
        val page = popular ?: return
        if (page.page >= page.totalPages) return
        busy = true
        scope.launch {
            try {
                val next = withContext(Dispatchers.IO) { logger.popularMovies(page.page + 1) }
                popular = SearchPage(
                    next.page,
                    page.results + next.results,
                    next.totalPages,
                    next.totalResults,
                )
            } catch (e: LoggerException) {
                onError(e.message ?: "popular films failed")
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
                selected = withContext(Dispatchers.IO) {
                    SelectedFilm.fromMovie(logger.findImdb(imdbQuery.trim()))
                }
                stage = AddStage.CONFIRM
            } catch (e: LoggerException) {
                onError(e.message ?: "film not found")
            } finally {
                busy = false
            }
        }
    }

    fun doAdd() {
        val s = selected ?: return
        val chosenRating = rating ?: run {
            onError("Choose a rating")
            return
        }
        val parsed = TimeUtils.fromLocalInput(watchedAtInput, TimeUtils.zone(timezone))
        if (parsed.isEmpty()) {
            onError("Invalid date or time zone, use yyyy-MM-dd HH:mm")
            return
        }
        val f = format ?: FormatText.build(FormatParts.forVenue(venue))
        val details = NewLogDetails(
            watchedAt = parsed,
            timezone = timezone,
            notes = null,
            rating = chosenRating,
        )
        busy = true
        scope.launch {
            try {
                withContext(Dispatchers.IO) {
                    val filmId = s.filmId
                    if (filmId != null) {
                        logger.logFilm(filmId, venue, f, details)
                    } else {
                        logger.addFilm(s.id, venue, f, details)
                    }
                }
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
            .verticalScroll(rememberScrollState())
            .padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
    ) {
        val s = selected
        if (s == null) {
            // Search stage: title search + IMDB finder. The confirm and
            // data-entry stages drop these fields.
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
            val page = results ?: popular
            if (page != null) {
                Text(
                    if (results == null) "Popular right now" else "Results",
                    style = MaterialTheme.typography.titleMedium,
                )
                Spacer(Modifier.height(8.dp))
                ResultsList(
                    page,
                    busy = busy,
                    onSelect = { r ->
                        selected = SelectedFilm.fromSummary(r)
                        stage = AddStage.CONFIRM
                    },
                )
                if (page.page < page.totalPages) {
                    OutlinedButton(
                        onClick = { if (results == null) loadMorePopular() else loadMore() },
                        enabled = !busy,
                    ) { Text("Load more") }
                }
            }
        } else {
            when (stage) {
                AddStage.CONFIRM ->
                    ConfirmCard(
                        film = s,
                        busy = busy,
                        onBack = { selected = null },
                        onConfirm = { stage = AddStage.FORMAT },
                    )

                AddStage.FORMAT ->
                    FormatCard(
                        catalog = catalog,
                        film = s,
                        venue = venue,
                        onVenue = { venue = it },
                        format = format,
                        onFormat = { format = it },
                        busy = busy,
                        onNext = { stage = AddStage.DETAILS },
                        onBack = {
                            if (preselected != null) {
                                selected = null
                            } else {
                                stage = AddStage.CONFIRM
                            }
                        },
                    )

                AddStage.DETAILS ->
                    DetailCard(
                        film = s,
                        timezone = timezone,
                        onTimezone = { timezone = it },
                        watchedAtInput = watchedAtInput,
                        onWatchedAt = { watchedAtInput = it },
                        rating = rating,
                        onRating = { rating = it },
                        busy = busy,
                        onLog = { doAdd() },
                        onBack = { stage = AddStage.FORMAT },
                    )
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
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        page.results.forEach { r ->
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
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val poster = posterUrl(film.posterPath)
            if (poster != null) {
                AsyncImage(
                    model = poster,
                    contentDescription = null,
                    modifier = Modifier
                        .width(48.dp)
                        .height(72.dp)
                        .clip(RoundedCornerShape(6.dp)),
                    contentScale = ContentScale.Crop,
                )
            }
            Spacer(Modifier.width(14.dp))
            Column(Modifier.weight(1f)) {
                Text(film.title, style = MaterialTheme.typography.titleMedium)
                Text(
                    film.year?.let { "($it)" } ?: "Year unknown",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun FormatCard(
    catalog: FormatCatalog,
    film: SelectedFilm,
    venue: Venue,
    onVenue: (Venue) -> Unit,
    format: Format?,
    onFormat: (Format) -> Unit,
    busy: Boolean,
    onNext: () -> Unit,
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
                .padding(16.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.Top,
            ) {
                val poster = posterUrl(film.posterPath)
                if (poster != null) {
                    AsyncImage(
                        model = poster,
                        contentDescription = null,
                        modifier = Modifier
                            .width(80.dp)
                            .height(120.dp)
                            .clip(RoundedCornerShape(10.dp)),
                        contentScale = ContentScale.FillBounds,
                    )
                    Spacer(Modifier.width(16.dp))
                }
                Column(Modifier.weight(1f)) {
                    Text(
                        if (film.year != null) "${film.title} (${film.year})" else film.title,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        "Step 1 of 2 · Venue and format",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
            Spacer(Modifier.height(18.dp))

            FormatFields(
                catalog = catalog,
                venue = venue,
                onVenue = onVenue,
                format = format,
                onFormat = onFormat,
            )
            Spacer(Modifier.height(16.dp))

            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                TextButton(onClick = onBack) { Text("‹ Back") }
                Spacer(Modifier.weight(1f))
                Button(onClick = onNext, enabled = !busy) { Text("Next") }
            }
        }
    }
}

@Composable
private fun DetailCard(
    film: SelectedFilm,
    timezone: String,
    onTimezone: (String) -> Unit,
    watchedAtInput: String,
    onWatchedAt: (String) -> Unit,
    rating: Rating?,
    onRating: (Rating) -> Unit,
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
                .padding(16.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.Top,
            ) {
                val poster = posterUrl(film.posterPath)
                if (poster != null) {
                    AsyncImage(
                        model = poster,
                        contentDescription = null,
                        modifier = Modifier
                            .width(80.dp)
                            .height(120.dp)
                            .clip(RoundedCornerShape(10.dp)),
                        contentScale = ContentScale.FillBounds,
                    )
                    Spacer(Modifier.width(16.dp))
                }
                Column(Modifier.weight(1f)) {
                    Text(
                        if (film.year != null) "${film.title} (${film.year})" else film.title,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        "Step 2 of 2 · Viewing details",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
            Spacer(Modifier.height(18.dp))

            ViewingFields(
                timezone = timezone,
                onTimezone = onTimezone,
                watchedAtInput = watchedAtInput,
                onWatchedAt = onWatchedAt,
                rating = rating,
                onRating = onRating,
            )
            Spacer(Modifier.height(16.dp))

            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                TextButton(onClick = onBack) { Text("‹ Back") }
                Spacer(Modifier.weight(1f))
                Button(
                    onClick = onLog,
                    enabled = !busy && rating != null,
                ) {
                    Text(if (film.filmId != null) "Log this viewing" else "Log this film")
                }
            }
        }
    }
}

/**
 * The confirmation step: shows the picked film (poster, title, year, IMDB
 * id when known) so the user can verify it before entering details.
 */
@Composable
private fun ConfirmCard(
    film: SelectedFilm,
    busy: Boolean,
    onBack: () -> Unit,
    onConfirm: () -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.medium,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
        ) {
            Row {
                val poster = posterUrl(film.posterPath)
                if (poster != null) {
                    AsyncImage(
                        model = poster,
                        contentDescription = null,
                        modifier = Modifier
                            .width(80.dp)
                            .height(120.dp)
                            .clip(RoundedCornerShape(10.dp)),
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
                    Spacer(Modifier.height(4.dp))
                    Text(
                        "Is this the film you watched?",
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
            Spacer(Modifier.height(12.dp))
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                TextButton(onClick = onBack) { Text("‹ Back") }
                Spacer(Modifier.weight(1f))
                Button(onClick = onConfirm, enabled = !busy) { Text("Confirm") }
            }
        }
    }
}
