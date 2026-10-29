package com.jakelandau.filmlogger.ui

import android.util.Log
import java.io.File
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import uniffi.film_logger_ffi.Logger
import uniffi.film_logger_ffi.LoggerException
import uniffi.film_logger_ffi.RatingBoard
import uniffi.film_logger_ffi.Stats
import uniffi.film_logger_ffi.WatchLog

private const val TABS_LOGS = 0
private const val TABS_ADD = 1
private const val TABS_STATS = 2
private const val TABS_SETTINGS = 3

/** The whole app: logger lifetime, bottom navigation, and the active view. */
@Composable
fun MainScreen() {
    val context = LocalContext.current
    var logger by remember { mutableStateOf<Logger?>(null) }
    var logs by remember { mutableStateOf<List<WatchLog>>(emptyList()) }
    var stats by remember { mutableStateOf<Stats?>(null) }
    var board by remember { mutableStateOf(RatingBoard.MPA) }
    var status by remember { mutableStateOf<String?>(null) }
    var tab by remember { mutableStateOf(TABS_LOGS) }
    var editingId by remember { mutableStateOf<Long?>(null) }

    fun refresh() {
        val l = logger ?: return
        logs = l.list()
        stats = l.stats()
        board = l.settings().selectedBoard
    }

    LaunchedEffect(Unit) {
        try {
            val db = File(context.filesDir, "film_logger.db")
            logger = Logger.open(db.absolutePath)
            refresh()
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
    val editingLog = editingId?.let { id -> logs.firstOrNull { it.id == id } }

    Surface(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            status?.let {
                Text(
                    it,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                    color = MaterialTheme.colorScheme.error,
                )
            }
            Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                when {
                    // `Logger.open` is still running (or failed: the status
                    // line above shows why) — no view may touch the logger
                    // while it is null.
                    l == null ->
                        Text(
                            if (status == null) "Loading…" else "Failed to open logger",
                            modifier = Modifier.padding(16.dp),
                        )

                    editingLog != null ->
                        EditView(
                            logger = l,
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
                                    onDelete = { id ->
                                        try {
                                            l.delete(id)
                                            refresh()
                                        } catch (e: LoggerException) {
                                            status = e.message
                                        }
                                    },
                                )

                            TABS_ADD ->
                                AddView(
                                    logger = l,
                                    onSaved = {
                                        tab = TABS_LOGS
                                        refresh()
                                    },
                                    onError = { status = it },
                                )

                            TABS_STATS -> stats?.let { StatsView(it) }

                            TABS_SETTINGS ->
                                SettingsView(
                                    logger = l,
                                    onSettingsChanged = { refresh() },
                                    onError = { status = it },
                                )
                        }
                }
            }
            Row(
                modifier =
                    Modifier
                        .padding(horizontal = 8.dp)
                        .padding(WindowInsets.navigationBars.asPaddingValues()),
            ) {
                NavTab("Logs", tab == TABS_LOGS, modifier = Modifier.weight(1f)) {
                    tab = TABS_LOGS
                    editingId = null
                }
                NavTab("Add", tab == TABS_ADD, modifier = Modifier.weight(1f)) {
                    tab = TABS_ADD
                    editingId = null
                }
                NavTab("Stats", tab == TABS_STATS, modifier = Modifier.weight(1f)) {
                    tab = TABS_STATS
                    editingId = null
                }
                NavTab("Settings", tab == TABS_SETTINGS, modifier = Modifier.weight(1f)) {
                    tab = TABS_SETTINGS
                    editingId = null
                }
            }
        }
    }
}
