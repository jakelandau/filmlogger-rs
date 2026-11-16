package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
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
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.film_logger_ffi.Collection
import uniffi.film_logger_ffi.CollectionPart
import uniffi.film_logger_ffi.FilmListEntry
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.MovieSummary
import uniffi.film_logger_ffi.SearchPage

private enum class FilmTab { ALL, FAVORITES, BUCKET_LIST, COLLECTIONS }

/** The film library: every stored film, filterable and searchable. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun FilmsView(
    logger: Logger,
    films: List<FilmListEntry>,
    onOpen: (FilmListEntry) -> Unit,
    onOpenTmdb: (Long) -> Unit,
    onRefresh: () -> Unit,
    onError: (String) -> Unit,
) {
    var tab by remember { mutableStateOf(FilmTab.ALL) }
    var search by remember { mutableStateOf("") }
    var collections by remember { mutableStateOf<Map<Long, Collection>>(emptyMap()) }
    var loadingCollections by remember { mutableStateOf(false) }
    var tmdbResults by remember { mutableStateOf<SearchPage?>(null) }
    var searchedQuery by remember { mutableStateOf("") }
    var searchBusy by remember { mutableStateOf(false) }
    var addingTmdb by remember { mutableStateOf<Long?>(null) }
    val scope = rememberCoroutineScope()

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

    fun searchTmdb(page: Int = 1) {
        val query = search.trim()
        if (query.isEmpty()) return
        searchBusy = true
        scope.launch {
            try {
                val next = withContext(Dispatchers.IO) { logger.search(query, page) }
                tmdbResults = if (page == 1) {
                    next
                } else {
                    SearchPage(
                        next.page,
                        (tmdbResults?.results ?: emptyList()) + next.results,
                        next.totalPages,
                        next.totalResults,
                    )
                }
                searchedQuery = query
            } catch (e: LoggerException) {
                onError(e.message ?: "search failed")
            } finally {
                searchBusy = false
            }
        }
    }

    fun addToBucketList(tmdbId: Long) {
        if (addingTmdb != null) return
        addingTmdb = tmdbId
        scope.launch {
            try {
                withContext(Dispatchers.IO) { logger.addToWatchlist(tmdbId) }
                onRefresh()
            } catch (e: LoggerException) {
                onError(e.message ?: "bucket list update failed")
            } finally {
                addingTmdb = null
            }
        }
    }

    val needle = search.trim().lowercase()
    val visible = films.filter { entry ->
        val isListed = entry.viewings > 0u || entry.film.watchlisted
        when (tab) {
            FilmTab.ALL -> isListed
            FilmTab.FAVORITES -> entry.film.favorite && isListed
            FilmTab.BUCKET_LIST -> entry.film.watchlisted && entry.viewings == 0u
            FilmTab.COLLECTIONS -> true
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
                onValueChange = {
                    search = it
                    if (it.trim() != searchedQuery) tmdbResults = null
                },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Search films") },
                singleLine = true,
            )
            Spacer(Modifier.height(10.dp))
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
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
                    selected = tab == FilmTab.BUCKET_LIST,
                    onClick = { tab = FilmTab.BUCKET_LIST },
                    label = { Text("Bucket List") },
                )
                FilterChip(
                    selected = tab == FilmTab.COLLECTIONS,
                    onClick = { tab = FilmTab.COLLECTIONS },
                    label = { Text("Collections") },
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End,
            ) {
                TextButton(
                    onClick = { searchTmdb(1) },
                    enabled = search.isNotBlank() && !searchBusy,
                ) {
                    Text(if (searchBusy) "Searching…" else "Search TMDB")
                }
            }
        }

        if (tab == FilmTab.COLLECTIONS) {
            CollectionsList(
                films = films,
                collections = collections,
                loading = loadingCollections,
                search = needle,
                onOpen = onOpen,
                onOpenTmdb = onOpenTmdb,
            )
            return@Column
        }

        val libraryByTmdb = films
            .mapNotNull { entry -> entry.film.tmdbId?.let { it to entry } }
            .toMap()

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            tmdbResults?.let { page ->
                item {
                    Text(
                        "TMDB results for “$searchedQuery”",
                        style = MaterialTheme.typography.titleMedium,
                    )
                }
                if (page.results.isEmpty()) {
                    item {
                        Text(
                            "No TMDB results.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                items(page.results) { result ->
                    TmdbResultRow(
                        result = result,
                        entry = libraryByTmdb[result.id],
                        adding = addingTmdb == result.id,
                        onAdd = { addToBucketList(result.id) },
                        onOpen = { entry -> onOpen(entry) },
                    )
                }
                if (page.page < page.totalPages) {
                    item {
                        TextButton(
                            onClick = { searchTmdb(page.page + 1) },
                            enabled = !searchBusy,
                        ) {
                            Text(if (searchBusy) "Loading…" else "Load more")
                        }
                    }
                }
                if (visible.isNotEmpty()) {
                    item {
                        Text("In your library", style = MaterialTheme.typography.titleMedium)
                    }
                }
            }
            if (visible.isNotEmpty()) {
                items(visible) { entry -> FilmRow(entry, onClick = { onOpen(entry) }) }
            } else {
                item {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Text(
                            when {
                                films.isEmpty() -> "No films yet"
                                tmdbResults != null -> "No local matches"
                                else -> "No matching films"
                            },
                            style = MaterialTheme.typography.titleMedium,
                        )
                        if (films.isEmpty()) {
                            Spacer(Modifier.height(8.dp))
                            Text(
                                "Search TMDB or add a film from the Add tab.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }
        }

    }
}

@Composable
private fun TmdbResultRow(
    result: MovieSummary,
    entry: FilmListEntry?,
    adding: Boolean,
    onAdd: () -> Unit,
    onOpen: (FilmListEntry) -> Unit,
) {
    val seen = (entry?.viewings ?: 0u) > 0u
    val onBucketList = entry?.film?.watchlisted == true
    val status = when {
        seen -> "Seen"
        onBucketList -> "On Bucket List"
        entry != null -> "In library"
        else -> "Not in library"
    }
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shape = MaterialTheme.shapes.medium,
    ) {
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
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val poster = posterUrl(result.posterPath)
            if (poster != null) {
                AsyncImage(
                    model = poster,
                    contentDescription = null,
                    modifier = Modifier
                        .width(44.dp)
                        .height(66.dp)
                        .clip(RoundedCornerShape(5.dp)),
                )
            } else {
                Box(
                    modifier = Modifier
                        .width(44.dp)
                        .height(66.dp)
                        .clip(RoundedCornerShape(5.dp))
                        .background(MaterialTheme.colorScheme.surface),
                )
            }
            Spacer(Modifier.width(12.dp))
            Column(Modifier.weight(1f)) {
                val year = yearFrom(result.releaseDate)
                Text(
                    if (year != null) "${result.title} ($year)" else result.title,
                    style = MaterialTheme.typography.titleSmall,
                )
                Spacer(Modifier.height(2.dp))
                Text(
                    status,
                    style = MaterialTheme.typography.bodySmall,
                    color = if (onBucketList || seen) {
                        MaterialTheme.colorScheme.primary
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
                )
            }
            if (!seen && !onBucketList) {
                TextButton(onClick = onAdd, enabled = !adding) {
                    Text(if (adding) "Adding…" else "Bucket List")
                }
            }
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
    onOpenTmdb: (Long) -> Unit,
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
            CollectionCard(collection, libraryByTmdb, onOpen, onOpenTmdb)
        }
    }
}

@Composable
private fun CollectionCard(
    collection: Collection,
    libraryByTmdb: Map<Long, FilmListEntry>,
    onOpen: (FilmListEntry) -> Unit,
    onOpenTmdb: (Long) -> Unit,
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
                        .clickable {
                            if (entry != null) {
                                onOpen(entry)
                            } else {
                                onOpenTmdb(part.tmdbId)
                            }
                        }
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
