package ca.jakelandau.filmlogger.ui

import android.util.Log
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import ca.jakelandau.filmlogger.R
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.film_logger_ffi.FilmId
import uniffi.film_logger_ffi.FilmListEntry
import uniffi.film_logger_ffi.FilmMetadata
import uniffi.film_logger_ffi.FilmPage
import uniffi.film_logger_ffi.FormatCatalog
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.RatingBoard
import uniffi.film_logger_ffi.Stats
import uniffi.film_logger_ffi.WatchLog

private const val TABS_LOGS = 0
private const val TABS_FILMS = 1
private const val TABS_ADD = 2
private const val TABS_STATS = 3
private const val TABS_SETTINGS = 4

/** Parse a storage key (`tt…` / `tmdb:…`) back into a [`FilmId`]. */
fun filmIdFromKey(key: String): FilmId =
    if (key.startsWith("tmdb:")) FilmId.Tmdb(key.removePrefix("tmdb:").toLong()) else FilmId.Imdb(key)

/** The whole app: logger lifetime, bottom navigation, and the active view. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var logger by remember { mutableStateOf<Logger?>(null) }
    var catalog by remember { mutableStateOf<FormatCatalog?>(null) }
    var logs by remember { mutableStateOf<List<WatchLog>>(emptyList()) }
    var films by remember { mutableStateOf<List<FilmListEntry>>(emptyList()) }
    var stats by remember { mutableStateOf<Stats?>(null) }
    var board by remember { mutableStateOf("US") }
    var boards by remember { mutableStateOf<List<RatingBoard>>(emptyList()) }
    var status by remember { mutableStateOf<String?>(null) }
    var tab by remember { mutableStateOf(TABS_LOGS) }
    var editingId by remember { mutableStateOf<Long?>(null) }
    var filmId by remember { mutableStateOf<String?>(null) }
    var filmPage by remember { mutableStateOf<FilmPage?>(null) }
    var preselected by remember { mutableStateOf<FilmMetadata?>(null) }

    suspend fun refresh(l: Logger) {
        val triple = withContext(Dispatchers.IO) {
            Triple(l.list(), l.films(), l.stats())
        }
        logs = triple.first
        films = triple.second
        stats = triple.third
        board = withContext(Dispatchers.IO) { l.settings().selectedBoard }
        boards = withContext(Dispatchers.IO) { l.ratingBoards() }
    }

    fun refresh() {
        val l = logger ?: return
        scope.launch {
            try {
                refresh(l)
            } catch (t: Throwable) {
                status = t.message ?: t.toString()
            }
        }
    }

    fun openFilm(key: String) {
        val l = logger ?: return
        filmId = key
        filmPage = null
        scope.launch {
            try {
                val page = withContext(Dispatchers.IO) { l.filmPage(filmIdFromKey(key)) }
                if (filmId == key) filmPage = page
            } catch (t: Throwable) {
                status = t.message ?: t.toString()
            }
        }
    }

    LaunchedEffect(Unit) {
        try {
            val db = File(context.filesDir, "film_logger.db")
            val opened = withContext(Dispatchers.IO) { Logger.open(db.absolutePath) }
            val loadedCatalog = withContext(Dispatchers.IO) { opened.formatCatalog() }
            logger = opened
            catalog = loadedCatalog
            refresh(opened)
        } catch (t: Throwable) {
            // `LoggerException` carries the domain error; anything else (a
            // JNA `UnsatisfiedLinkError`, a Rust panic surfaced as
            // `InternalException`, …) would otherwise vanish with the
            // `logger` still `null`.
            Log.e("FilmLogger", "failed to open logger", t)
            status = t.message?.ifEmpty { t.toString() } ?: t.toString()
        }
    }

    val l = logger
    val cat = catalog
    val editingLog = editingId?.let { id -> logs.firstOrNull { it.id == id } }

    val screenTitle = when {
        l == null || cat == null -> "FilmLogger"
        editingLog != null -> "Edit viewing"
        filmId != null -> "Film"
        tab == TABS_ADD -> "Add film"
        tab == TABS_LOGS -> "FilmLogger"
        tab == TABS_FILMS -> "Films"
        tab == TABS_STATS -> "Stats"
        else -> "Settings"
    }
    val showBack = editingLog != null || filmId != null || tab == TABS_ADD
    val showBottomBar = editingLog == null && filmId == null && tab != TABS_ADD

    fun selectTab(next: Int) {
        tab = next
        editingId = null
        filmId = null
        filmPage = null
        preselected = null
    }

    fun goBack() {
        when {
            editingLog != null -> editingId = null
            filmId != null -> {
                filmId = null
                filmPage = null
            }
            tab != TABS_LOGS -> selectTab(TABS_LOGS)
        }
    }

    BackHandler(enabled = editingLog != null || filmId != null || tab != TABS_LOGS) {
        goBack()
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            CenterAlignedTopAppBar(
                title = {
                    Text(
                        screenTitle,
                        style = MaterialTheme.typography.headlineMedium,
                        maxLines = 1,
                    )
                },
                navigationIcon = {
                    if (showBack) {
                        IconButton(onClick = { goBack() }) {
                            Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                        }
                    }
                },
            )
        },
        bottomBar = {
            if (showBottomBar) {
                NavigationBar {
                    NavigationBarItem(
                        selected = tab == TABS_LOGS,
                        onClick = { selectTab(TABS_LOGS) },
                        icon = { Icon(Icons.AutoMirrored.Filled.List, contentDescription = null) },
                        label = { Text("Logs") },
                    )
                    NavigationBarItem(
                        selected = tab == TABS_FILMS,
                        onClick = { selectTab(TABS_FILMS) },
                        icon = {
                            Icon(
                                painter = painterResource(R.drawable.ic_film_strip),
                                contentDescription = null,
                            )
                        },
                        label = { Text("Films") },
                    )
                    NavigationBarItem(
                        selected = tab == TABS_STATS,
                        onClick = { selectTab(TABS_STATS) },
                        icon = { Icon(Icons.Filled.Info, contentDescription = null) },
                        label = { Text("Stats") },
                    )
                    NavigationBarItem(
                        selected = tab == TABS_SETTINGS,
                        onClick = { selectTab(TABS_SETTINGS) },
                        icon = { Icon(Icons.Filled.Settings, contentDescription = null) },
                        label = { Text("Settings") },
                    )
                }
            }
        },
        floatingActionButton = {
            if (l != null && cat != null && tab == TABS_LOGS && filmId == null && editingLog == null) {
                FloatingActionButton(
                    onClick = {
                        preselected = null
                        editingId = null
                        filmId = null
                        filmPage = null
                        tab = TABS_ADD
                    },
                ) {
                    Icon(Icons.Filled.Add, contentDescription = "Add a film")
                }
            }
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
        ) {
            status?.let {
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    shape = MaterialTheme.shapes.medium,
                    color = MaterialTheme.colorScheme.errorContainer,
                    contentColor = MaterialTheme.colorScheme.onErrorContainer,
                ) {
                    Text(
                        it,
                        modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
            Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                when {
                    // `Logger.open` is still running (or failed: the status
                    // line above shows why) — no view may touch the logger
                    // while it is null.
                    l == null || cat == null ->
                        Text(
                            if (status == null) "Loading…" else "Failed to open logger",
                            modifier = Modifier.padding(24.dp),
                            style = MaterialTheme.typography.titleMedium,
                        )

                    editingLog != null ->
                        EditView(
                            logger = l,
                            catalog = cat,
                            log = editingLog,
                            onDone = {
                                editingId = null
                                refresh()
                            },
                            onDeleted = {
                                editingId = null
                                refresh()
                            },
                            onError = { status = it },
                        )

                    filmId != null ->
                        FilmView(
                            logger = l,
                            page = filmPage,
                            selectedBoard = board,
                            onBack = {
                                filmId = null
                                filmPage = null
                            },
                            onLogAnother = { film ->
                                preselected = film
                                filmId = null
                                filmPage = null
                                tab = TABS_ADD
                            },
                            onChanged = { refresh() },
                            onReload = { filmId?.let { openFilm(it) } },
                            onError = { status = it },
                        )

                    else ->
                        when (tab) {
                            TABS_LOGS ->
                                LogsView(
                                    logs = logs,
                                    selectedBoard = board,
                                    onEdit = { id ->
                                        tab = TABS_LOGS
                                        editingId = id
                                    },
                                    onAdd = { tab = TABS_ADD },
                                )

                            TABS_FILMS ->
                                FilmsView(
                                    logger = l,
                                    films = films,
                                    onOpen = { entry -> openFilm(filmKeyOf(entry)) },
                                    onOpenTmdb = { tmdbId ->
                                        scope.launch {
                                            try {
                                                val film = withContext(Dispatchers.IO) {
                                                    l.ensureFilm(tmdbId)
                                                }
                                                refresh()
                                                openFilm(filmKeyOf(film))
                                            } catch (t: Throwable) {
                                                status = t.message ?: t.toString()
                                            }
                                        }
                                    },
                                    onRefresh = { refresh() },
                                    onError = { status = it },
                                )

                            TABS_ADD ->
                                AddView(
                                    logger = l,
                                    catalog = cat,
                                    preselected = preselected,
                                    onSaved = {
                                        preselected = null
                                        tab = TABS_LOGS
                                        refresh()
                                    },
                                    onError = { status = it },
                                )

                            TABS_STATS -> stats?.let { StatsView(it, cat) }

                            TABS_SETTINGS ->
                                SettingsView(
                                    logger = l,
                                    boards = boards,
                                    onSettingsChanged = { refresh() },
                                    onError = { status = it },
                                )
                        }
                }
            }
        }
    }
}

/** A library entry's storage key. */
fun filmKeyOf(entry: FilmListEntry): String = filmKeyOf(entry.film)

/** A film's storage key. */
fun filmKeyOf(film: FilmMetadata): String = when (val id = film.filmId) {
    is FilmId.Imdb -> id.id
    is FilmId.Tmdb -> "tmdb:${id.id}"
}
