package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.film_logger_ffi.Certifications
import uniffi.film_logger_ffi.FilmMetadata
import uniffi.film_logger_ffi.FilmPage
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException

/** One film's page: metadata, user state, statistics, and history. */
@Composable
fun FilmView(
    logger: Logger,
    page: FilmPage?,
    selectedBoard: String,
    onBack: () -> Unit,
    onReload: () -> Unit,
    onLogAnother: (FilmMetadata) -> Unit,
    onChanged: () -> Unit,
    onError: (String) -> Unit,
) {
    if (page == null) {
        Text("Loading…", modifier = Modifier.padding(16.dp))
        return
    }
    val film = page.film
    val scope = rememberCoroutineScope()

    fun mutate(action: suspend () -> Unit) {
        scope.launch {
            try {
                withContext(Dispatchers.IO) { action() }
                onChanged()
                onReload()
            } catch (e: LoggerException) {
                onError(e.message ?: "action failed")
            }
        }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Column {
                TextButton(onClick = onBack) { Text("← Films", style = MaterialTheme.typography.labelLarge) }
                Row {
                    val poster = posterUrl(film.posterPath)
                    if (poster != null) {
                        AsyncImage(
                            model = poster,
                            contentDescription = null,
                            modifier = Modifier
                                .width(112.dp)
                                .height(168.dp)
                                .clip(RoundedCornerShape(10.dp)),
                            contentScale = ContentScale.Crop,
                        )
                        Spacer(Modifier.width(16.dp))
                    }
                    Column(Modifier.weight(1f)) {
                        val year = film.releaseYear
                        Text(
                            if (year != null) "${film.title} ($year)" else film.title,
                            style = MaterialTheme.typography.headlineSmall,
                        )
                        val meta = buildList {
                            year?.let { add("$it") }
                            film.runtimeMinutes?.let { add("$it min") }
                            if (film.genres.isNotEmpty()) add(film.genres.joinToString { it.name })
                        }
                        if (meta.isNotEmpty()) {
                            Spacer(Modifier.height(6.dp))
                            Text(
                                meta.joinToString(" · "),
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        CertificationLine(film.certifications, selectedBoard)
                    }
                }
                Spacer(Modifier.height(16.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    OutlinedButton(
                        onClick = { mutate { logger.setFavorite(film.filmId, !film.favorite) } },
                        modifier = Modifier.weight(1f),
                    ) {
                        Text(
                            if (film.favorite) "★ Favorite" else "☆ Favorite",
                            style = MaterialTheme.typography.labelLarge,
                        )
                    }
                    Button(
                        onClick = { onLogAnother(film) },
                        modifier = Modifier.weight(1f),
                    ) { Text("Log viewing", style = MaterialTheme.typography.labelLarge) }
                }
                film.tagline?.let {
                    Spacer(Modifier.height(12.dp))
                    Text(
                        it,
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
                film.overview?.let {
                    Spacer(Modifier.height(10.dp))
                    Text(it, style = MaterialTheme.typography.bodyMedium)
                }
                film.collection?.let {
                    Spacer(Modifier.height(10.dp))
                    Text(
                        "Collection: ${it.name}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (film.cast.isNotEmpty()) {
                    Spacer(Modifier.height(10.dp))
                    Text(
                        "Cast: " + film.cast.take(10).joinToString { c ->
                            c.character?.let { role -> "${c.name} as $role" } ?: c.name
                        },
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }

                Spacer(Modifier.height(16.dp))
                OutlinedButton(
                    onClick = { mutate { logger.refreshFilm(film.filmId) } },
                    modifier = Modifier.fillMaxWidth(),
                ) { Text("Refresh Metadata", style = MaterialTheme.typography.labelLarge) }
                Spacer(Modifier.height(18.dp))
                val stats = page.stats
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    shape = MaterialTheme.shapes.large,
                    color = MaterialTheme.colorScheme.surfaceVariant,
                ) {
                    Column(Modifier.padding(16.dp)) {
                        Text("Viewings", style = MaterialTheme.typography.titleLarge)
                        Spacer(Modifier.height(6.dp))
                        Text(
                            "${stats.viewings.toInt()} total · ${stats.theatre.toInt()} theatre · " +
                                "${stats.home.toInt()} home",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        stats.firstWatchedAt?.let {
                            Spacer(Modifier.height(4.dp))
                            Text(
                                "First ${TimeUtils.display(it)}",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        stats.lastWatchedAt?.let {
                            Spacer(Modifier.height(4.dp))
                            Text(
                                "Last ${TimeUtils.display(it)}",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        if (stats.formats.isNotEmpty()) {
                            Spacer(Modifier.height(8.dp))
                            Text(
                                stats.formats.joinToString(" · ") { "${it.key} × ${it.count.toInt()}" },
                                style = MaterialTheme.typography.bodyMedium,
                            )
                        }
                    }
                }

                Spacer(Modifier.height(18.dp))
                Text("History", style = MaterialTheme.typography.titleLarge)
            }
        }

        items(page.logs) { log ->
            val zone = log.timezone?.let { TimeUtils.zone(it) } ?: TimeUtils.systemZone()
            Surface(
                modifier = Modifier.fillMaxWidth(),
                color = MaterialTheme.colorScheme.surfaceVariant,
                shape = MaterialTheme.shapes.large,
            ) {
                Column(Modifier.padding(16.dp)) {
                    Text(
                        TimeUtils.display(log.watchedAt, zone) +
                            if (log.rewatch) "  · Rewatch" else "",
                        style = MaterialTheme.typography.titleSmall,
                    )
                    Spacer(Modifier.height(4.dp))
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        FormatFamilyBadge(log.format)
                        if (log.format != null) Spacer(Modifier.width(6.dp))
                        Text(
                            log.formatSummary.ifEmpty { "—" },
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    if (formatFlags(log.format).isNotEmpty()) {
                        Spacer(Modifier.height(4.dp))
                        FormatFlagsRow(log.format)
                    }
                    log.rating?.let {
                        Spacer(Modifier.height(4.dp))
                        Text(
                            if (it == uniffi.film_logger_ffi.Rating.UP) "👍 Thumbs up" else "👎 Thumbs down",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
        }

        item { Spacer(Modifier.height(12.dp)) }
    }
}

@Composable
private fun CertificationLine(certs: Certifications?, region: String) {
    val value = certs?.values?.firstOrNull { it.region == region }
    Spacer(Modifier.height(6.dp))
    Row(verticalAlignment = Alignment.CenterVertically) {
        if (value != null) {
            CertificationIcon(value.region, value.rating)
            Spacer(Modifier.width(6.dp))
            Text(
                "${value.region} ${value.rating}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            Text(
                "Unrated",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
