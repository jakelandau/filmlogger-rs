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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
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
import uniffi.filmlogger_rs_ffi.Collection
import uniffi.filmlogger_rs_ffi.CollectionPart
import uniffi.filmlogger_rs_ffi.FilmListEntry
import uniffi.filmlogger_rs_ffi.Logger
import uniffi.filmlogger_rs_ffi.LoggerException
import uniffi.filmlogger_rs_ffi.MovieSummary
import uniffi.filmlogger_rs_ffi.SearchPage

private val filmTabs = listOf(
    FilmTab.SEEN to "Seen",
    FilmTab.BUCKET_LIST to "Bucket List",
    FilmTab.COLLECTIONS to "Collections",
)

/** The film library: seen films, the bucket list, and collection progress. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun FilmsView(
    logger: Logger,
    films: List<FilmListEntry>,
    dataEpoch: Int,
    onOpen: (FilmListEntry) -> Unit,
    onOpenTmdb: (Long) -> Unit,
    onAdd: () -> Unit,
    onRefresh: () -> Unit,
    onError: (String) -> Unit,
) {
    var tab by remember { mutableStateOf(FilmTab.SEEN) }
    var favoritesOnly by remember { mutableStateOf(false) }
    var search by remember { mutableStateOf("") }
    var collections by remember(dataEpoch) { mutableStateOf<Map<Long, Collection>>(emptyMap()) }
    var loadingCollections by remember { mutableStateOf(false) }
    var tmdbResults by remember { mutableStateOf<SearchPage?>(null) }
    var searchedQuery by remember { mutableStateOf("") }
    var searchBusy by remember { mutableStateOf(false) }
    var addingTmdb by remember { mutableStateOf<Long?>(null) }
    var addingCollection by remember { mutableStateOf<Long?>(null) }
    val scope = rememberCoroutineScope()

    val collectionIds = films.mapNotNull { it.film.collection?.id }.distinct()
    LaunchedEffect(collectionIds, tab, dataEpoch) {
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

    val libraryByTmdb = remember(films) { filmLibraryByTmdb(films) }

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

    fun addToBucketList(tmdbId: Long, stored: FilmListEntry? = null) {
        if (addingTmdb != null || addingCollection != null) return
        addingTmdb = tmdbId
        scope.launch {
            try {
                withContext(Dispatchers.IO) {
                    if (stored != null) {
                        logger.setWatchlisted(stored.film.filmId, true)
                    } else {
                        logger.addToWatchlist(tmdbId)
                    }
                }
                onRefresh()
            } catch (e: LoggerException) {
                onError(e.message ?: "bucket list update failed")
            } finally {
                addingTmdb = null
            }
        }
    }

    // Add every part that is neither seen nor watchlisted; individual failures
    // do not abort the run and are reported once at the end.
    fun addMissingToBucketList(collection: Collection) {
        if (addingTmdb != null || addingCollection != null) return
        addingCollection = collection.id
        val missing = missingCollectionParts(collection, libraryByTmdb)
        scope.launch {
            val failures = try {
                withContext(Dispatchers.IO) {
                    var failed = 0
                    for (part in missing) {
                        val stored = libraryByTmdb[part.tmdbId]
                        try {
                            if (stored != null) {
                                logger.setWatchlisted(stored.film.filmId, true)
                            } else {
                                logger.addToWatchlist(part.tmdbId)
                            }
                        } catch (e: LoggerException) {
                            failed += 1
                        }
                    }
                    failed
                }
            } catch (e: LoggerException) {
                onError(e.message ?: "bucket list update failed")
                -1
            } finally {
                addingCollection = null
            }
            onRefresh()
            if (failures > 0) {
                onError("$failures of ${missing.size} films could not be added to the bucket list")
            }
        }
    }

    val needle = search.trim().lowercase()
    val visible = visibleLibraryFilms(films, tab, favoritesOnly, search)

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
                label = {
                    Text(if (tab == FilmTab.COLLECTIONS) "Search collections" else "Search films")
                },
                singleLine = true,
            )
            Spacer(Modifier.height(10.dp))
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                for ((value, label) in filmTabs) {
                    FilterChip(
                        selected = tab == value,
                        onClick = { tab = value },
                        label = { Text(label) },
                    )
                }
                if (tab == FilmTab.SEEN) {
                    FilterChip(
                        selected = favoritesOnly,
                        onClick = { favoritesOnly = !favoritesOnly },
                        label = { Text("Favorites") },
                    )
                }
            }
            if (tab != FilmTab.COLLECTIONS) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                ) {
                    Button(
                        onClick = { searchTmdb(1) },
                        enabled = search.isNotBlank() && !searchBusy,
                    ) {
                        Icon(Icons.Filled.Search, contentDescription = null)
                        Spacer(Modifier.width(8.dp))
                        Text(if (searchBusy) "Searching…" else "Search TMDB")
                    }
                }
            }
        }

        if (tab == FilmTab.COLLECTIONS) {
            CollectionsList(
                films = films,
                collections = collections,
                loading = loadingCollections,
                search = needle,
                libraryByTmdb = libraryByTmdb,
                addingTmdb = addingTmdb,
                addingCollection = addingCollection,
                onOpen = onOpen,
                onOpenTmdb = onOpenTmdb,
                onAddPart = { tmdbId, stored -> addToBucketList(tmdbId, stored) },
                onAddMissing = { addMissingToBucketList(it) },
            )
            return@Column
        }

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
                        onAdd = { addToBucketList(result.id, libraryByTmdb[result.id]) },
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
                        Text(
                            if (tab == FilmTab.SEEN) "Seen" else "Bucket List",
                            style = MaterialTheme.typography.titleMedium,
                        )
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
                                tmdbResults != null -> "No local matches."
                                needle.isNotEmpty() -> "No films match this filter."
                                tab == FilmTab.SEEN && favoritesOnly -> "No favorite films seen."
                                tab == FilmTab.SEEN -> "No films seen yet."
                                else ->
                                    "Your bucket list is empty. Add films from TMDB search " +
                                        "or from a collection."
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
                            Spacer(Modifier.height(8.dp))
                            TextButton(onClick = onAdd) {
                                Text("Add your first film")
                            }
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
    libraryByTmdb: Map<Long, FilmListEntry>,
    addingTmdb: Long?,
    addingCollection: Long?,
    onOpen: (FilmListEntry) -> Unit,
    onOpenTmdb: (Long) -> Unit,
    onAddPart: (Long, FilmListEntry?) -> Unit,
    onAddMissing: (Collection) -> Unit,
) {
    val grouped = films
        .mapNotNull { it.film.collection }
        .distinctBy { it.id }
        .mapNotNull { ref -> collections[ref.id] }
    val filtered = grouped.filter { collection ->
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
            Text("No collections match this search.", style = MaterialTheme.typography.titleMedium)
        }
        return
    }

    val completed = grouped.count { isCollectionComplete(it, libraryByTmdb) }
    val ordered = filtered.sortedWith(
        compareBy<Collection> { if (isCollectionComplete(it, libraryByTmdb)) 1 else 0 }
            .thenBy { it.name.lowercase() },
    )

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        if (search.isEmpty()) {
            item {
                Text(
                    "$completed of ${grouped.size} collections completed",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        items(ordered) { collection ->
            CollectionCard(
                collection = collection,
                libraryByTmdb = libraryByTmdb,
                addingTmdb = addingTmdb,
                addingCollection = addingCollection,
                onOpen = onOpen,
                onOpenTmdb = onOpenTmdb,
                onAddPart = onAddPart,
                onAddMissing = onAddMissing,
            )
        }
    }
}

@Composable
private fun CompletedBadge() {
    Surface(
        color = MaterialTheme.colorScheme.primary,
        shape = MaterialTheme.shapes.small,
    ) {
        Text(
            "Completed",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp),
        )
    }
}

@Composable
private fun CollectionCard(
    collection: Collection,
    libraryByTmdb: Map<Long, FilmListEntry>,
    addingTmdb: Long?,
    addingCollection: Long?,
    onOpen: (FilmListEntry) -> Unit,
    onOpenTmdb: (Long) -> Unit,
    onAddPart: (Long, FilmListEntry?) -> Unit,
    onAddMissing: (Collection) -> Unit,
) {
    val parts = collection.parts.sortedWith(
        compareBy<CollectionPart> { it.releaseYear ?: Int.MAX_VALUE }
            .thenBy { it.title.lowercase() },
    )
    val total = parts.size
    val seen = seenCount(collection, libraryByTmdb)
    val complete = isCollectionComplete(collection, libraryByTmdb)
    val missing = missingCollectionParts(collection, libraryByTmdb)
    val busy = addingCollection != null || addingTmdb != null
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
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            collection.name,
                            style = MaterialTheme.typography.titleLarge,
                            modifier = Modifier.weight(1f, fill = false),
                        )
                        if (complete) {
                            Spacer(Modifier.width(8.dp))
                            CompletedBadge()
                        }
                    }
                    if (total > 0) {
                        Spacer(Modifier.height(4.dp))
                        Text(
                            "$seen of $total seen",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                if (total > 0) {
                    Spacer(Modifier.width(12.dp))
                    Box(
                        modifier = Modifier.size(40.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        CircularProgressIndicator(
                            progress = { seen.toFloat() / total },
                            modifier = Modifier.fillMaxSize(),
                            strokeWidth = 4.dp,
                            trackColor = MaterialTheme.colorScheme.surface,
                        )
                        Text(
                            "${Math.round(seen * 100f / total)}%",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            if (missing.isNotEmpty()) {
                TextButton(
                    onClick = { onAddMissing(collection) },
                    enabled = !busy,
                    modifier = Modifier.align(Alignment.End),
                ) {
                    Text(
                        if (addingCollection == collection.id) "Adding…"
                        else "Add missing to Bucket List",
                    )
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
                val seenPart = entry?.viewings?.let { it > 0u } == true
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
                        if (seenPart) "✓" else "○",
                        style = MaterialTheme.typography.titleMedium,
                        color = if (seenPart) {
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
                    when {
                        seenPart -> Text(
                            "Seen",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                        entry?.film?.watchlisted == true -> Text(
                            "On Bucket List",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                        else -> TextButton(
                            onClick = { onAddPart(part.tmdbId, entry) },
                            enabled = !busy,
                        ) {
                            Text(
                                if (addingTmdb == part.tmdbId) "Adding…" else "Bucket List",
                            )
                        }
                    }
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
                    if (entry.viewings == 0u) {
                        Text(
                            "On Bucket List",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                    } else {
                        Text(
                            "${entry.viewings.toInt()} viewing" +
                                if (entry.viewings == 1u) "" else "s",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        if (film.favorite) {
                            Spacer(Modifier.width(8.dp))
                            Text(
                                "★ Favorite",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.primary,
                            )
                        }
                    }
                }
            }
        }
    }
}
