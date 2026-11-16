package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.film_logger_ffi.Collection
import uniffi.film_logger_ffi.CollectionPart
import uniffi.film_logger_ffi.FilmListEntry
import uniffi.film_logger_ffi.Logger

private enum class FilmTab { ALL, FAVORITES, COLLECTIONS }

/** The film library: every stored film, filterable and searchable. */
@Composable
fun FilmsView(
    logger: Logger,
    films: List<FilmListEntry>,
    onOpen: (FilmListEntry) -> Unit,
) {
    var tab by remember { mutableStateOf(FilmTab.ALL) }
    var search by remember { mutableStateOf("") }
    var collections by remember { mutableStateOf<Map<Long, Collection>>(emptyMap()) }
    var loadingCollections by remember { mutableStateOf(false) }

    val collectionIds = films.mapNotNull { it.film.collection?.id }.distinct()
    LaunchedEffect(collectionIds, tab) {
        if (tab != FilmTab.COLLECTIONS) return@LaunchedEffect
        val missing = collectionIds.filter { it !in collections }
        if (missing.isEmpty()) return@LaunchedEffect
        loadingCollections = true
        try {
            val fetched = withContext(Dispatchers.IO) {
                missing.mapNotNull { id ->
                    runCatching { id to logger.collection(id) }.getOrNull()
                }.toMap()
            }
            collections = collections + fetched
        } finally {
            loadingCollections = false
        }
    }

    val needle = search.trim().lowercase()
    val visible = films.filter { entry ->
        when (tab) {
            FilmTab.ALL, FilmTab.COLLECTIONS -> true
            FilmTab.FAVORITES -> entry.film.favorite
        } &&
            (needle.isEmpty() ||
                entry.film.title.lowercase().contains(needle))
    }

    Column(Modifier.fillMaxSize()) {
        Column(
            Modifier.padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 12.dp),
        ) {
            OutlinedTextField(
                value = search,
                onValueChange = { search = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Search films") },
                singleLine = true,
            )
            Spacer(Modifier.height(10.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilterChip(
                    selected = tab == FilmTab.ALL,
                    onClick = { tab = FilmTab.ALL },
                    label = { Text("All") },
                )
                FilterChip(
                    selected = tab == FilmTab.FAVORITES,
                    onClick = { tab = FilmTab.FAVORITES },
                    label = { Text("Favorites") },
                )
                FilterChip(
                    selected = tab == FilmTab.COLLECTIONS,
                    onClick = { tab = FilmTab.COLLECTIONS },
                    label = { Text("Collections") },
                )
            }
        }

        if (films.isEmpty()) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(24.dp),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text("No films yet", style = MaterialTheme.typography.titleLarge)
                Spacer(Modifier.height(8.dp))
                Text(
                    "Add a film from the Add tab to build your library.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            return@Column
        }
        if (tab == FilmTab.COLLECTIONS) {
            CollectionsList(
                films = films,
                collections = collections,
                loading = loadingCollections,
                search = needle,
                onOpen = onOpen,
            )
            return@Column
        }

        if (visible.isEmpty()) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(24.dp),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text("No matching films", style = MaterialTheme.typography.titleLarge)
                Spacer(Modifier.height(8.dp))
                Text(
                    "Try a different search or filter.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            return@Column
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            items(visible) { entry -> FilmRow(entry, onClick = { onOpen(entry) }) }
        }
    }
}

@Composable
private fun CollectionsList(
    films: List<FilmListEntry>,
    collections: Map<Long, Collection>,
    loading: Boolean,
    search: String,
    onOpen: (FilmListEntry) -> Unit,
) {
    val libraryByTmdb = remember(films) {
        films.mapNotNull { entry ->
            entry.film.tmdbId?.let { it to entry }
        }.toMap()
    }
    val grouped = remember(films, collections) {
        films
            .mapNotNull { it.film.collection }
            .distinctBy { it.id }
            .mapNotNull { ref -> collections[ref.id]?.let { ref to it } }
    }
    val filtered = grouped.filter { (_, collection) ->
        search.isEmpty() ||
            collection.name.lowercase().contains(search) ||
            collection.parts.any { it.title.lowercase().contains(search) }
    }

    if (grouped.isEmpty()) {
        Column(
            modifier = Modifier.fillMaxSize().padding(24.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                if (loading) "Loading collections…" else "No films in a collection yet.",
                style = MaterialTheme.typography.titleMedium,
            )
        }
        return
    }
    if (filtered.isEmpty()) {
        Column(
            modifier = Modifier.fillMaxSize().padding(24.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text("No matching collections", style = MaterialTheme.typography.titleMedium)
        }
        return
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        items(filtered) { (_, collection) ->
            CollectionCard(collection, libraryByTmdb, onOpen)
        }
    }
}

@Composable
private fun CollectionCard(
    collection: Collection,
    libraryByTmdb: Map<Long, FilmListEntry>,
    onOpen: (FilmListEntry) -> Unit,
) {
    val parts = collection.parts.sortedWith(
        compareBy<CollectionPart> { it.releaseYear ?: Int.MAX_VALUE }
            .thenBy { it.title.lowercase() },
    )
    val seenCount = parts.count { part ->
        libraryByTmdb[part.tmdbId]?.viewings?.let { it > 0u } == true
    }
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.large,
    ) {
        Column(Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.Top) {
                val posterPath = collection.posterPath ?: parts.firstOrNull()?.posterPath
                val poster = posterUrl(posterPath)
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
                Spacer(Modifier.width(14.dp))
                Column(Modifier.weight(1f)) {
                    Text(collection.name, style = MaterialTheme.typography.titleLarge)
                    if (parts.isNotEmpty()) {
                        Spacer(Modifier.height(4.dp))
                        Text(
                            "$seenCount of ${parts.size} seen",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            Spacer(Modifier.height(10.dp))
            if (parts.isEmpty()) {
                Text(
                    "No films returned for this collection.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            for (part in parts) {
                val entry = libraryByTmdb[part.tmdbId]
                val seen = entry?.viewings?.let { it > 0u } == true
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .then(
                            if (entry != null) {
                                Modifier.clickable { onOpen(entry) }
                            } else {
                                Modifier
                            }
                        )
                        .padding(vertical = 6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        if (seen) "✓" else "○",
                        style = MaterialTheme.typography.titleMedium,
                        color = if (seen) {
                            MaterialTheme.colorScheme.primary
                        } else {
                            MaterialTheme.colorScheme.onSurfaceVariant
                        },
                    )
                    Spacer(Modifier.width(10.dp))
                    Column(Modifier.weight(1f)) {
                        Text(part.title, style = MaterialTheme.typography.bodyMedium)
                        Text(
                            part.releaseYear?.toString() ?: "Year unknown",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Text(
                        if (seen) "Seen" else "Not seen",
                        style = MaterialTheme.typography.labelSmall,
                        color = if (seen) {
                            MaterialTheme.colorScheme.primary
                        } else {
                            MaterialTheme.colorScheme.onSurfaceVariant
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun FilmRow(entry: FilmListEntry, onClick: () -> Unit) {
    val film = entry.film
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.large)
            .clickable(onClick = onClick),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.large,
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
            Spacer(Modifier.width(16.dp))
            Column(Modifier.weight(1f)) {
                val year = film.releaseYear
                Text(
                    if (year != null) "${film.title} ($year)" else film.title,
                    style = MaterialTheme.typography.titleMedium,
                )
                Spacer(Modifier.height(4.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        "${entry.viewings.toInt()} viewing${if (entry.viewings == 1u) "" else "s"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    if (film.favorite) {
                        Spacer(Modifier.width(8.dp))
                        Text("★ Favorite", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
                    }
                }
            }
        }
    }
}
