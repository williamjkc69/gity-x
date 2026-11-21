#[cfg(target_os = "macos")]
pub const MOD_KEY: &str = "Cmd";
#[cfg(not(target_os = "macos"))]
pub const MOD_KEY: &str = "Ctrl";

#[cfg(target_os = "macos")]
macro_rules! with_mod {
    ($pre:expr, $post:expr) => {
        concat!($pre, "Cmd", $post)
    };
}
#[cfg(not(target_os = "macos"))]
macro_rules! with_mod {
    ($pre:expr, $post:expr) => {
        concat!($pre, "Ctrl", $post)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Es,
}

impl Default for Language {
    fn default() -> Self {
        Self::En
    }
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s.trim() {
            "es" => Self::Es,
            _ => Self::En,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Español",
        }
    }
}

pub struct Strings {
    pub app_subtitle: &'static str,
    pub open_repo: &'static str,
    pub open_repo_btn: &'static str,
    pub open: &'static str,
    pub paste_repo_hint: &'static str,
    pub recent_repos: &'static str,
    pub all_branches: &'static str,
    pub fetch: &'static str,
    pub pull: &'static str,
    pub push: &'static str,
    pub new_branch: &'static str,
    pub stash: &'static str,
    pub pop: &'static str,
    pub refresh: &'static str,
    pub settings: &'static str,
    pub search_hint: &'static str,
    pub local_branches: &'static str,
    pub remote_branches: &'static str,
    pub tags: &'static str,
    pub stashes: &'static str,
    pub no_stashes: &'static str,
    pub sidebar_tip: &'static str,
    pub col_graph: &'static str,
    pub col_commit: &'static str,
    pub col_author: &'static str,
    pub col_date: &'static str,
    pub col_hash: &'static str,
    pub no_commits: &'static str,
    pub select_prompt: &'static str,
    pub wip_title: &'static str,
    pub unstaged: &'static str,
    pub staged: &'static str,
    pub stage_all: &'static str,
    pub unstage_all: &'static str,
    pub no_pending_files: &'static str,
    pub no_staged_files: &'static str,
    pub discard_hover: &'static str,
    pub stage_hover: &'static str,
    pub unstage_hover: &'static str,
    pub commit_msg_label: &'static str,
    pub commit_msg_hint: &'static str,
    pub amend_last: &'static str,
    pub commit_btn: &'static str,
    pub amend_btn: &'static str,
    pub parents: &'static str,
    pub checkout: &'static str,
    pub cherry_pick: &'static str,
    pub revert: &'static str,
    pub rebase_onto: &'static str,
    pub branch_here: &'static str,
    pub tag_here: &'static str,
    pub compare_select: &'static str,
    pub compare_with_selected: &'static str,
    pub compare_with_working: &'static str,
    pub clear_compare: &'static str,
    pub comparing_commits: &'static str,
    pub comparing_working: &'static str,
    pub reset_soft: &'static str,
    pub reset_mixed: &'static str,
    pub reset_hard: &'static str,
    pub copy_hash: &'static str,
    pub copy_message: &'static str,
    pub copy_name: &'static str,
    pub changed_files: &'static str,
    pub view_all: &'static str,
    pub view_tree: &'static str,
    pub view_list: &'static str,
    pub full_diff: &'static str,
    pub no_diff: &'static str,
    pub merge_into_current: &'static str,
    pub merge_no_ff: &'static str,
    pub rename_branch: &'static str,
    pub rename_branch_title: &'static str,
    pub rename_btn: &'static str,
    pub push_branch: &'static str,
    pub delete_branch: &'static str,
    pub force_delete_branch: &'static str,
    pub delete_remote_branch: &'static str,
    pub push_tag: &'static str,
    pub delete_tag: &'static str,
    pub stash_apply: &'static str,
    pub stash_pop: &'static str,
    pub stash_drop: &'static str,
    pub create_branch_title: &'static str,
    pub create_branch_from: &'static str,
    pub from_commit: &'static str,
    pub base_ref_label: &'static str,
    pub current_head: &'static str,
    pub branch_name: &'static str,
    pub toggle_left_sidebar: &'static str,
    pub toggle_right_sidebar: &'static str,
    pub create_tag_title: &'static str,
    pub tag_name: &'static str,
    pub tag_message_opt: &'static str,
    pub stash_modal_title: &'static str,
    pub stash_msg_opt: &'static str,
    pub cancel: &'static str,
    pub create_and_checkout: &'static str,
    pub create: &'static str,
    pub save_stash: &'static str,
    pub settings_title: &'static str,
    pub language_label: &'static str,
    pub commit_limit_label: &'static str,
    pub graph_style_label: &'static str,
    pub graph_rounded: &'static str,
    pub graph_angular: &'static str,
    pub commit_order_label: &'static str,
    pub order_topo: &'static str,
    pub order_date: &'static str,
    pub order_author: &'static str,
    pub show_remotes_label: &'static str,
    pub compact_rows_label: &'static str,
    pub auto_check_remote_label: &'static str,
    pub close: &'static str,
    pub continue_op: &'static str,
    pub abort_op: &'static str,
    pub state_cherry_pick: &'static str,
    pub state_merge: &'static str,
    pub state_rebase: &'static str,
    pub msg_refreshed: &'static str,
    pub msg_fetched: &'static str,
    pub msg_pulled: &'static str,
    pub msg_pushed: &'static str,
    pub msg_committed: &'static str,
    pub msg_stashed: &'static str,
    pub msg_stash_popped: &'static str,
    pub msg_cherry_picked: &'static str,
    pub msg_reverted: &'static str,
    pub msg_hash_copied: &'static str,
    pub msg_message_copied: &'static str,
    pub msg_name_copied: &'static str,
    pub mode_graph: &'static str,
    pub mode_agent: &'static str,
    pub agent_review_title: &'static str,
    pub agent_untracked: &'static str,
    pub agent_clean_title: &'static str,
    pub agent_clean_sub: &'static str,
    pub agent_split: &'static str,
    pub agent_unified: &'static str,
    pub agent_all_files: &'static str,
    pub agent_single_file: &'static str,
    pub agent_accept: &'static str,
    pub agent_accepted: &'static str,
    pub agent_revert: &'static str,
    pub agent_quick_commit_hint: &'static str,
    pub maximize_changes: &'static str,
    pub error_modal_title: &'static str,
    pub error_details_label: &'static str,
    pub copy_btn: &'static str,
    pub copied_btn: &'static str,
    pub mode_prs: &'static str,
    pub pr_open: &'static str,
    pub pr_merged: &'static str,
    pub pr_closed: &'static str,
    pub pr_all: &'static str,
    pub pr_draft: &'static str,
    pub pr_filter_hint: &'static str,
    pub pr_loading: &'static str,
    pub pr_none_found: &'static str,
    pub pr_no_provider: &'static str,
    pub pr_configure_token: &'static str,
    pub pr_tab_conversation: &'static str,
    pub pr_tab_commits: &'static str,
    pub pr_tab_files: &'static str,
    pub pr_open_browser: &'static str,
    pub pr_checkout_branch: &'static str,
    pub pr_merge_commit: &'static str,
    pub pr_squash_merge: &'static str,
    pub pr_rebase_merge: &'static str,
    pub pr_merge_btn: &'static str,
    pub pr_approve_btn: &'static str,
    pub pr_request_changes_btn: &'static str,
    pub pr_comment_btn: &'static str,
    pub pr_close_decline_btn: &'static str,
    pub pr_reopen_btn: &'static str,
    pub pr_comment_hint: &'static str,
    pub pr_no_description: &'static str,
    pub pr_approved_badge: &'static str,
    pub pr_changes_requested_badge: &'static str,
    pub auto_update_label: &'static str,
    pub check_updates_btn: &'static str,
    pub update_checking: &'static str,
    pub update_up_to_date: &'static str,
    pub update_available_btn: &'static str,
    pub update_downloading: &'static str,
    pub update_restart_btn: &'static str,
    pub help_center_title: &'static str,
    pub help_btn: &'static str,
    pub troubleshoot_btn: &'static str,
    pub settings_tab_general: &'static str,
    pub settings_tab_providers: &'static str,
    pub test_token_btn: &'static str,
    pub testing_token: &'static str,
    pub show_tokens: &'static str,
    pub hide_tokens: &'static str,
    pub pr_click_to_expand: &'static str,
    pub pr_token_missing_banner: &'static str,
}

pub const EN: Strings = Strings {
    app_subtitle: "Visual Git commit graph & fast desktop client in Rust",
    open_repo: "Open Repo",
    open_repo_btn: "Open Git Repository...",
    open: "Open",
    paste_repo_hint: "Or paste a repository path here...",
    recent_repos: "RECENT REPOSITORIES",
    all_branches: "All Branches",
    fetch: "Fetch",
    pull: "Pull",
    push: "Push",
    new_branch: "+ Branch",
    stash: "Stash",
    pop: "Pop",
    refresh: "Refresh",
    settings: "Settings",
    search_hint: with_mod!("Search commits, author, hash, ref... (", "+F)"),
    local_branches: "LOCAL",
    remote_branches: "REMOTES",
    tags: "TAGS",
    stashes: "STASHES",
    no_stashes: "No stashes",
    sidebar_tip: "Click: locate in graph\nDouble-click: checkout • Right-click: actions",
    col_graph: "GRAPH",
    col_commit: "COMMIT / REFS",
    col_author: "AUTHOR",
    col_date: "DATE",
    col_hash: "HASH",
    no_commits: "This repository has no commits yet.",
    select_prompt: "Select a commit or WIP row\nto inspect details and diffs.",
    wip_title: "Uncommitted Changes (WIP)",
    unstaged: "UNSTAGED",
    staged: "STAGED",
    stage_all: "Stage all",
    unstage_all: "Unstage all",
    no_pending_files: "No unstaged files",
    no_staged_files: "No staged files",
    discard_hover: "Discard changes",
    stage_hover: "Stage file",
    unstage_hover: "Unstage file",
    commit_msg_label: "Commit message",
    commit_msg_hint: "Write commit message...",
    amend_last: "Amend last commit",
    commit_btn: "Commit",
    amend_btn: "Amend Commit",
    parents: "Parents:",
    checkout: "Checkout",
    cherry_pick: "Cherry-pick",
    revert: "Revert",
    rebase_onto: "Rebase current branch onto this",
    branch_here: "+ Branch",
    tag_here: "+ Tag",
    compare_select: "Select for compare",
    compare_with_selected: "Compare with selected",
    compare_with_working: "Compare with Working Tree",
    clear_compare: "Clear compare",
    comparing_commits: "Comparing commits",
    comparing_working: "Comparing with Working Tree",
    reset_soft: "Reset current branch here (Soft)",
    reset_mixed: "Reset current branch here (Mixed)",
    reset_hard: "Reset current branch here (Hard)",
    copy_hash: "Copy full hash",
    copy_message: "Copy commit message",
    copy_name: "Copy name",
    changed_files: "CHANGED FILES",
    view_all: "All",
    view_tree: "Tree",
    view_list: "List",
    full_diff: "FULL DIFF",
    no_diff: "No diff to display.",
    merge_into_current: "Merge into current branch",
    merge_no_ff: "Merge into current (--no-ff)",
    rename_branch: "Rename branch...",
    rename_branch_title: "Rename Branch",
    rename_btn: "Rename",
    push_branch: "Push branch to origin",
    delete_branch: "Delete branch (-d)",
    force_delete_branch: "Force delete branch (-D)",
    delete_remote_branch: "Delete remote branch",
    push_tag: "Push tag to origin",
    delete_tag: "Delete tag",
    stash_apply: "Apply stash",
    stash_pop: "Pop stash",
    stash_drop: "Drop stash",
    create_branch_title: "Create Branch From...",
    create_branch_from: "Create branch from here...",
    from_commit: "From commit:",
    base_ref_label: "Base (branch, tag or commit):",
    current_head: "Current HEAD",
    branch_name: "New branch name:",
    toggle_left_sidebar: with_mod!("Toggle left sidebar (", "+B)"),
    toggle_right_sidebar: with_mod!("Toggle inspector panel (", "+I)"),
    create_tag_title: "Create Tag",
    tag_name: "Tag name:",
    tag_message_opt: "Message (optional, annotated tag):",
    stash_modal_title: "Create Stash",
    stash_msg_opt: "Stash message (optional):",
    cancel: "Cancel",
    create_and_checkout: "Create & Checkout",
    create: "Create",
    save_stash: "Save Stash",
    settings_title: "Settings — gity x",
    language_label: "Language",
    commit_limit_label: "Max commits loaded",
    graph_style_label: "Graph line style",
    graph_rounded: "Rounded",
    graph_angular: "Angular",
    commit_order_label: "Commit ordering",
    order_topo: "Topo",
    order_date: "Date",
    order_author: "Author",
    show_remotes_label: "Show remote branches in graph",
    compact_rows_label: "Compact graph rows",
    auto_check_remote_label: "Auto-check remote updates (git ls-remote)",
    close: "Close",
    continue_op: "Continue",
    abort_op: "Abort",
    state_cherry_pick: "Cherry-pick in progress",
    state_merge: "Merge in progress",
    state_rebase: "Rebase in progress",
    msg_refreshed: "Repository refreshed.",
    msg_fetched: "Fetch completed.",
    msg_pulled: "Pull completed.",
    msg_pushed: "Push completed.",
    msg_committed: "Commit created.",
    msg_stashed: "Changes saved to stash.",
    msg_stash_popped: "Stash applied and dropped.",
    msg_cherry_picked: "Cherry-pick applied to current branch.",
    msg_reverted: "Commit reverted.",
    msg_hash_copied: "Hash copied to clipboard.",
    msg_message_copied: "Commit message copied.",
    msg_name_copied: "Name copied to clipboard.",
    mode_graph: "Git Graph",
    mode_agent: "Agent Mode",
    agent_review_title: "Working Changes",
    agent_untracked: "UNTRACKED",
    agent_clean_title: "Working tree is clean",
    agent_clean_sub: with_mod!("No uncommitted files, untracked files, or stashes selected. Press Refresh (", "+R) anytime."),
    agent_split: "Split",
    agent_unified: "Unified",
    agent_all_files: "All Files",
    agent_single_file: "Focused",
    agent_accept: "Stage",
    agent_accepted: "Unstage",
    agent_revert: "Discard",
    agent_quick_commit_hint: "Commit message...",
    maximize_changes: with_mod!("Maximize changes view (Agent Mode — ", "+E)"),
    error_modal_title: "Git Error",
    error_details_label: "Full output",
    copy_btn: "Copy",
    copied_btn: "✓ Copied",
    mode_prs: "Pull Requests",
    pr_open: "Open",
    pr_merged: "Merged",
    pr_closed: "Closed",
    pr_all: "All",
    pr_draft: "Draft",
    pr_filter_hint: "Filter PRs by title, #number, author...",
    pr_loading: "Loading...",
    pr_none_found: "No pull requests match this filter.",
    pr_no_provider: "No supported Git hosting provider (GitHub, GitLab, Bitbucket) detected in remotes.",
    pr_configure_token: "Configure Token",
    pr_tab_conversation: "Conversation",
    pr_tab_commits: "Commits",
    pr_tab_files: "Files changed",
    pr_open_browser: "Open in browser",
    pr_checkout_branch: "Checkout branch",
    pr_merge_commit: "Create a merge commit",
    pr_squash_merge: "Squash and merge",
    pr_rebase_merge: "Rebase and merge",
    pr_merge_btn: "Merge",
    pr_approve_btn: "✓ Approve",
    pr_request_changes_btn: "Request changes",
    pr_comment_btn: "Comment",
    pr_close_decline_btn: "Close / Decline",
    pr_reopen_btn: "Reopen",
    pr_comment_hint: "Leave a comment or review note...",
    pr_no_description: "No description provided.",
    pr_approved_badge: "✓ Approved",
    pr_changes_requested_badge: "× Changes requested",
    auto_update_label: "Auto-download app updates on startup",
    check_updates_btn: "Check for updates",
    update_checking: "Checking updates...",
    update_up_to_date: "Up to date",
    update_available_btn: "Update",
    update_downloading: "Updating",
    update_restart_btn: "Restart to update",
    help_center_title: "Help & Troubleshooting Center",
    help_btn: "Help",
    troubleshoot_btn: "Troubleshoot",
    settings_tab_general: "General & Graph",
    settings_tab_providers: "Git Providers & Tokens",
    test_token_btn: "Test",
    testing_token: "Testing...",
    show_tokens: "Show",
    hide_tokens: "Hide",
    pr_click_to_expand: "Click any Pull Request to expand details, conversation, commits, and diffs.",
    pr_token_missing_banner: "No API token detected — private repos return HTTP 404 without authentication.",
};

pub const ES: Strings = Strings {
    app_subtitle: "Visualizador de árbol Git y cliente rápido en Rust",
    open_repo: "Abrir Repo",
    open_repo_btn: "Abrir repositorio Git...",
    open: "Abrir",
    paste_repo_hint: "O pega la ruta del repositorio aquí...",
    recent_repos: "REPOSITORIOS RECIENTES",
    all_branches: "Todas las ramas",
    fetch: "Fetch",
    pull: "Pull",
    push: "Push",
    new_branch: "+ Rama",
    stash: "Stash",
    pop: "Pop",
    refresh: "Actualizar",
    settings: "Ajustes",
    search_hint: with_mod!("Buscar commits, autor, hash, rama... (", "+F)"),
    local_branches: "LOCALES",
    remote_branches: "REMOTAS",
    tags: "TAGS",
    stashes: "STASHES",
    no_stashes: "Sin stashes",
    sidebar_tip: "Click: ubicar en árbol\nDoble click: checkout • Click der.: acciones",
    col_graph: "ÁRBOL",
    col_commit: "COMMIT / RAMAS",
    col_author: "AUTOR",
    col_date: "FECHA",
    col_hash: "HASH",
    no_commits: "Este repositorio aún no tiene commits.",
    select_prompt: "Selecciona un commit o fila WIP\npara ver detalles y diffs.",
    wip_title: "Cambios sin confirmar (WIP)",
    unstaged: "SIN STAGE",
    staged: "EN STAGE",
    stage_all: "Stage todo",
    unstage_all: "Quitar todo",
    no_pending_files: "Sin archivos pendientes",
    no_staged_files: "Ningún archivo en stage",
    discard_hover: "Descartar cambios",
    stage_hover: "Agregar al stage",
    unstage_hover: "Quitar del stage",
    commit_msg_label: "Mensaje del commit",
    commit_msg_hint: "Escribe el mensaje del commit...",
    amend_last: "Enmendar último commit (amend)",
    commit_btn: "Commit",
    amend_btn: "Enmendar Commit",
    parents: "Padres:",
    checkout: "Checkout",
    cherry_pick: "Cherry-pick",
    revert: "Revert",
    rebase_onto: "Rebase rama actual sobre esto",
    branch_here: "+ Rama",
    tag_here: "+ Tag",
    compare_select: "Seleccionar para comparar",
    compare_with_selected: "Comparar con seleccionado",
    compare_with_working: "Comparar con Working Tree",
    clear_compare: "Limpiar comparación",
    comparing_commits: "Comparando commits",
    comparing_working: "Comparando con Working Tree",
    reset_soft: "Reset rama actual aquí (Soft)",
    reset_mixed: "Reset rama actual aquí (Mixed)",
    reset_hard: "Reset rama actual aquí (Hard)",
    copy_hash: "Copiar hash completo",
    copy_message: "Copiar mensaje",
    copy_name: "Copiar nombre",
    changed_files: "ARCHIVOS MODIFICADOS",
    view_all: "Todos",
    view_tree: "Árbol",
    view_list: "Lista",
    full_diff: "DIFF COMPLETO",
    no_diff: "Sin cambios para mostrar.",
    merge_into_current: "Merge en rama actual",
    merge_no_ff: "Merge en rama actual (--no-ff)",
    rename_branch: "Renombrar rama...",
    rename_branch_title: "Renombrar rama",
    rename_btn: "Renombrar",
    push_branch: "Push rama a origin",
    delete_branch: "Eliminar rama (-d)",
    force_delete_branch: "Forzar eliminar rama (-D)",
    delete_remote_branch: "Eliminar rama remota",
    push_tag: "Push tag a origin",
    delete_tag: "Eliminar tag",
    stash_apply: "Aplicar stash",
    stash_pop: "Pop stash",
    stash_drop: "Eliminar stash",
    create_branch_title: "Crear rama desde...",
    create_branch_from: "Crear rama desde aquí...",
    from_commit: "Desde commit:",
    base_ref_label: "Base (rama, tag o commit):",
    current_head: "HEAD actual",
    branch_name: "Nombre de la nueva rama:",
    toggle_left_sidebar: with_mod!("Mostrar/ocultar panel izquierdo (", "+B)"),
    toggle_right_sidebar: with_mod!("Mostrar/ocultar inspector (", "+I)"),
    create_tag_title: "Crear tag",
    tag_name: "Nombre del tag:",
    tag_message_opt: "Mensaje (opcional, tag anotado):",
    stash_modal_title: "Crear Stash",
    stash_msg_opt: "Mensaje del stash (opcional):",
    cancel: "Cancelar",
    create_and_checkout: "Crear y cambiar",
    create: "Crear",
    save_stash: "Guardar Stash",
    settings_title: "Configuración — gity x",
    language_label: "Idioma",
    commit_limit_label: "Máximo de commits cargados",
    graph_style_label: "Estilo de líneas del árbol",
    graph_rounded: "Curvas",
    graph_angular: "Angular",
    commit_order_label: "Orden de commits",
    order_topo: "Topo",
    order_date: "Fecha",
    order_author: "Autor",
    show_remotes_label: "Mostrar ramas remotas en el árbol",
    compact_rows_label: "Filas compactas en el árbol",
    auto_check_remote_label: "Comprobar cambios remotos automáticamente",
    close: "Cerrar",
    continue_op: "Continuar",
    abort_op: "Abortar",
    state_cherry_pick: "Cherry-pick en progreso",
    state_merge: "Merge en progreso",
    state_rebase: "Rebase en progreso",
    msg_refreshed: "Repositorio actualizado.",
    msg_fetched: "Fetch completado.",
    msg_pulled: "Pull completado.",
    msg_pushed: "Push completado.",
    msg_committed: "Commit creado.",
    msg_stashed: "Cambios guardados en stash.",
    msg_stash_popped: "Stash aplicado y eliminado.",
    msg_cherry_picked: "Cherry-pick aplicado a la rama actual.",
    msg_reverted: "Commit revertido.",
    msg_hash_copied: "Hash copiado al portapapeles.",
    msg_message_copied: "Mensaje del commit copiado.",
    msg_name_copied: "Nombre copiado al portapapeles.",
    mode_graph: "Git Graph",
    mode_agent: "Agent Mode",
    agent_review_title: "Cambios Locales",
    agent_untracked: "SIN SEGUIMIENTO (UNTRACKED)",
    agent_clean_title: "Sin cambios pendientes",
    agent_clean_sub: with_mod!("No hay archivos sin confirmar, untracked ni stashes seleccionados. Usa Actualizar (", "+R)."),
    agent_split: "Dividido",
    agent_unified: "Unificado",
    agent_all_files: "Todos",
    agent_single_file: "Archivo",
    agent_accept: "Stage",
    agent_accepted: "Unstage",
    agent_revert: "Descartar",
    agent_quick_commit_hint: "Mensaje del commit...",
    maximize_changes: with_mod!("Maximizar vista de cambios (Agent Mode — ", "+E)"),
    error_modal_title: "Error de Git",
    error_details_label: "Detalle completo",
    copy_btn: "Copiar",
    copied_btn: "✓ Copiado",
    mode_prs: "Pull Requests",
    pr_open: "Abiertos",
    pr_merged: "Merged",
    pr_closed: "Cerrados",
    pr_all: "Todos",
    pr_draft: "Borrador",
    pr_filter_hint: "Filtrar PRs por título, #número, autor...",
    pr_loading: "Cargando...",
    pr_none_found: "No hay pull requests en este filtro.",
    pr_no_provider: "No se detectó un proveedor compatible (GitHub, GitLab, Bitbucket) en los remotos.",
    pr_configure_token: "Configurar Token",
    pr_tab_conversation: "Conversación",
    pr_tab_commits: "Commits",
    pr_tab_files: "Archivos",
    pr_open_browser: "Abrir en navegador",
    pr_checkout_branch: "Checkout rama",
    pr_merge_commit: "Merge commit",
    pr_squash_merge: "Squash y merge",
    pr_rebase_merge: "Rebase y merge",
    pr_merge_btn: "Merge",
    pr_approve_btn: "✓ Aprobar",
    pr_request_changes_btn: "Pedir cambios",
    pr_comment_btn: "Comentar",
    pr_close_decline_btn: "Cerrar / Declinar",
    pr_reopen_btn: "Reabrir",
    pr_comment_hint: "Escribe un comentario o revisión...",
    pr_no_description: "Sin descripción.",
    pr_approved_badge: "✓ Aprobado",
    pr_changes_requested_badge: "× Cambios solicitados",
    auto_update_label: "Descargar actualizaciones automáticamente al abrir",
    check_updates_btn: "Buscar actualizaciones",
    update_checking: "Buscando actualizaciones...",
    update_up_to_date: "Actualizado",
    update_available_btn: "Actualizar",
    update_downloading: "Actualizando",
    update_restart_btn: "Reiniciar para actualizar",
    help_center_title: "Centro de Ayuda y Solución de Problemas",
    help_btn: "Ayuda",
    troubleshoot_btn: "Solucionar",
    settings_tab_general: "General y Grafo",
    settings_tab_providers: "Proveedores Git y Tokens",
    test_token_btn: "Probar",
    testing_token: "Probando...",
    show_tokens: "Mostrar",
    hide_tokens: "Ocultar",
    pr_click_to_expand: "Haz clic en cualquier Pull Request para expandir detalles, conversación, commits y diffs.",
    pr_token_missing_banner: "No se detectó token de API — los repos privados devuelven HTTP 404 sin autenticación.",
};

pub fn tr(lang: Language) -> &'static Strings {
    match lang {
        Language::En => &EN,
        Language::Es => &ES,
    }
}

/// Formats a git `%ai` date (`YYYY-MM-DD HH:MM:SS ±HHMM`) into a clean human-readable string.
pub fn format_human_date(iso: &str, lang: Language) -> String {
    let mut parts = iso.split_whitespace();
    let Some(date_part) = parts.next() else {
        return iso.to_string();
    };
    let Some(time_part) = parts.next() else {
        return iso.to_string();
    };

    let d: Vec<&str> = date_part.split('-').collect();
    let t: Vec<&str> = time_part.split(':').collect();
    if d.len() < 3 || t.len() < 2 {
        return iso.to_string();
    }

    let year = d[0];
    let month: usize = d[1].parse().unwrap_or(0);
    let day: usize = d[2].parse().unwrap_or(0);
    let hour24: usize = t[0].parse().unwrap_or(0);
    let minute = t[1];

    let (hour12, ampm) = match hour24 {
        0 => (12, "AM"),
        1..=11 => (hour24, "AM"),
        12 => (12, "PM"),
        _ => (hour24 - 12, "PM"),
    };

    const MONTHS_EN: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const MONTHS_ES: [&str; 12] = [
        "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
    ];

    if !(1..=12).contains(&month) || day == 0 {
        return iso.to_string();
    }

    match lang {
        Language::En => format!(
            "{} {}, {} · {}:{minute} {ampm}",
            MONTHS_EN[month - 1],
            day,
            year,
            hour12
        ),
        Language::Es => format!(
            "{} {} {} · {}:{minute} {ampm}",
            day,
            MONTHS_ES[month - 1],
            year,
            hour12
        ),
    }
}

/// Translates common cryptic git errors into a clear summary + actionable hint.
pub fn humanize_git_error(raw: &str, lang: Language) -> (&'static str, &'static str) {
    let lower = raw.to_lowercase();

    if lower.contains("would be overwritten") || lower.contains("local changes") {
        return match lang {
            Language::En => (
                "Local uncommitted changes would be overwritten",
                "Commit or Stash your changes before switching branches or pulling.",
            ),
            Language::Es => (
                "Hay cambios locales que se sobrescribirían",
                "Haz Commit o Stash de tus cambios antes de cambiar de rama o hacer Pull.",
            ),
        };
    }

    if lower.contains("conflict") || lower.contains("could not apply") {
        return match lang {
            Language::En => (
                "Conflict detected during operation",
                "Resolve conflicted files and press Continue, or click Abort in the top banner.",
            ),
            Language::Es => (
                "Se detectó un conflicto en la operación",
                "Resuelve los archivos en conflicto y pulsa Continuar, o usa Abortar en la barra superior.",
            ),
        };
    }

    if lower.contains("not fully merged") {
        return match lang {
            Language::En => (
                "Branch is not fully merged",
                "Use 'Force delete branch (-D)' from the right-click menu if you still want to delete it.",
            ),
            Language::Es => (
                "La rama tiene commits sin fusionar",
                "Usa 'Forzar eliminar rama (-D)' en el menú contextual si deseas eliminarla igualmente.",
            ),
        };
    }

    if lower.contains("non-fast-forward")
        || lower.contains("fetch first")
        || lower.contains("[rejected]")
    {
        return match lang {
            Language::En => (
                "Push rejected — remote has newer commits",
                "Run Pull (or Fetch) to integrate remote changes before pushing.",
            ),
            Language::Es => (
                "Push rechazado — el remoto tiene commits más recientes",
                "Ejecuta Pull (o Fetch) para integrar los cambios remotos antes de subir.",
            ),
        };
    }

    if lower.contains("not a git repository") {
        return match lang {
            Language::En => (
                "Selected folder is not a Git repository",
                "Choose a directory that contains a .git folder or initialize it first.",
            ),
            Language::Es => (
                "La carpeta seleccionada no es un repositorio Git",
                "Elige un directorio que contenga .git o inicialízalo primero.",
            ),
        };
    }

    if lower.contains("already exists") {
        return match lang {
            Language::En => (
                "Name already exists",
                "Choose a different branch or tag name.",
            ),
            Language::Es => (
                "El nombre ya existe",
                "Elige un nombre diferente para la rama o tag.",
            ),
        };
    }

    if lower.contains("authentication failed")
        || lower.contains("permission denied")
        || lower.contains("could not read from remote")
    {
        return match lang {
            Language::En => (
                "Remote authentication failed",
                "Verify your SSH keys, access token, or network connection.",
            ),
            Language::Es => (
                "Falló la autenticación con el repositorio remoto",
                "Verifica tus llaves SSH, token de acceso o conexión a internet.",
            ),
        };
    }

    if lower.contains("no upstream branch") || lower.contains("has no upstream") {
        return match lang {
            Language::En => (
                "Branch has no upstream remote configured",
                "Right-click the branch in the sidebar and choose 'Push branch to origin'.",
            ),
            Language::Es => (
                "La rama no tiene upstream remoto configurado",
                "Haz click derecho en la rama en el panel izquierdo y usa 'Push rama a origin'.",
            ),
        };
    }

    if lower.contains("http 401") || lower.contains("bad credentials") {
        return match lang {
            Language::En => (
                "API token rejected (HTTP 401 Unauthorized)",
                "Your token is invalid or expired. Open Settings → Git Providers & Tokens, paste a fresh token, and click Test.",
            ),
            Language::Es => (
                "Token de API rechazado (HTTP 401 No autorizado)",
                "Tu token es inválido o expiró. Abre Configuración → Proveedores Git y Tokens, pega uno válido y pulsa Probar.",
            ),
        };
    }

    if lower.contains("saml") || lower.contains("sso") {
        return match lang {
            Language::En => (
                "Organization SAML SSO authorization required",
                "In GitHub Settings → Developer settings → Personal access tokens, click 'Configure SSO' next to your token and authorize the organization.",
            ),
            Language::Es => (
                "Se requiere autorización SAML SSO de la organización",
                "En GitHub → Developer settings → Personal access tokens, pulsa 'Configure SSO' junto a tu token y autoriza la organización.",
            ),
        };
    }

    if lower.contains("approve your own pull request")
        || lower.contains("can not approve your own")
        || lower.contains("cannot approve your own")
    {
        return match lang {
            Language::En => (
                "Cannot approve your own Pull Request",
                "GitHub and Bitbucket prevent PR authors from approving their own PR. Another reviewer must approve it, or you can Merge directly if allowed.",
            ),
            Language::Es => (
                "No puedes aprobar tu propio Pull Request",
                "GitHub y Bitbucket impiden que el autor apruebe su propio PR. Debe aprobarlo otro revisor, o haz Merge directo si tienes permisos.",
            ),
        };
    }

    if lower.contains("http 403") || lower.contains("rate limit") {
        return match lang {
            Language::En => (
                "Permission denied or rate limit exceeded (HTTP 403)",
                "Check required token scopes (GitHub: 'repo' / Pull requests R&W; GitLab: 'api'; Bitbucket: Pull requests Write) in Settings → Git Providers, or open Troubleshoot.",
            ),
            Language::Es => (
                "Permiso denegado o límite excedido (HTTP 403)",
                "Verifica los permisos del token (GitHub: 'repo' o Pull requests R/W; GitLab: 'api'; Bitbucket: Pull requests Write) en Configuración o abre Solucionar.",
            ),
        };
    }

    if lower.contains("http 404") && lower.contains("api error") {
        return match lang {
            Language::En => (
                "Repository not found or private without authentication (HTTP 404)",
                "Private repos return HTTP 404 when no valid token has access. Open Settings → Git Providers & Tokens, configure your token, and click Test.",
            ),
            Language::Es => (
                "Repositorio no encontrado o privado sin autenticación (HTTP 404)",
                "Los repos privados devuelven HTTP 404 sin un token con acceso. Abre Configuración → Proveedores Git y Tokens, configura tu token y pulsa Probar.",
            ),
        };
    }

    if lower.contains("http 405") || lower.contains("http 409") || lower.contains("not mergeable") {
        return match lang {
            Language::En => (
                "Pull Request cannot be merged right now",
                "Check for merge conflicts, failing CI checks, or branch protection rules requiring reviews before merging.",
            ),
            Language::Es => (
                "El Pull Request no se puede fusionar en este momento",
                "Revisa si hay conflictos, checks de CI fallando o reglas de protección de rama que exigen aprobación.",
            ),
        };
    }

    if lower.contains("nothing to commit") || lower.contains("no local changes to save") {
        return match lang {
            Language::En => (
                "Nothing to save or commit",
                "Your working tree has no matching changes for this operation.",
            ),
            Language::Es => (
                "No hay cambios para guardar o confirmar",
                "Tu árbol de trabajo no tiene cambios pendientes para esta acción.",
            ),
        };
    }

    match lang {
        Language::En => (
            "Git operation failed",
            "Review the full output below or click Copy to debug.",
        ),
        Language::Es => (
            "La operación de Git falló",
            "Revisa el detalle completo abajo o pulsa Copiar para depurar.",
        ),
    }
}
