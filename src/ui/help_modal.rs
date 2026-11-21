use eframe::egui::{self, Align, Color32, Layout, Margin, RichText, Rounding, Stroke};

use crate::app::{GitKrakenApp, SettingsTab};
use crate::i18n::Language;
use crate::theme;

struct HelpTopic {
    category: &'static str,
    title: &'static str,
    symptom: &'static str,
    steps: &'static [&'static str],
    command: Option<&'static str>,
    link_label: Option<&'static str>,
    link_url: Option<&'static str>,
    open_providers_settings: bool,
}

const TOPICS_EN: &[HelpTopic] = &[
    HelpTopic {
        category: "GitHub",
        title: "GitHub PRs return HTTP 404 / 401 / 403 (Private Repos, Scopes, SAML SSO)",
        symptom: "GitHub hides private repositories with HTTP 404 when no authorized token is sent, or HTTP 403 if scopes/SSO are missing.",
        steps: &[
            "1. Recommended: Sign in once with GitHub CLI ('gh auth login') — gity x detects it automatically on macOS, Linux, and Windows.",
            "2. Or create a Classic Personal Access Token with the 'repo' scope checked and paste it in Settings → Git Providers & Tokens → GitHub.",
            "3. If using a Fine-grained PAT: select the target repositories AND grant 'Pull requests: Read and write' + 'Contents: Read'.",
            "4. Organization repos using SAML SSO: open GitHub Personal Access Tokens, click 'Configure SSO' next to your token, and click 'Authorize'.",
            "5. Click 'Test' in Settings to verify your username and active scopes immediately.",
        ],
        command: Some("gh auth login && gh auth status"),
        link_label: Some("Create GitHub Token"),
        link_url: Some("https://github.com/settings/tokens/new?scopes=repo&description=gity-x"),
        open_providers_settings: true,
    },
    HelpTopic {
        category: "Bitbucket",
        title: "Bitbucket PRs return HTTP 404 / 401 / 403 (API Token vs Access Token)",
        symptom: "Bitbucket Cloud returns HTTP 404 for private repositories without valid authentication, or 401 if Basic vs Bearer auth is mismatched.",
        steps: &[
            "Option A — Atlassian Account API Token (works across all your workspaces): create an API token at id.atlassian.com, enter your Atlassian account EMAIL in 'Bitbucket User' and the API token in 'Bitbucket Token'.",
            "Option B — Repository / Workspace Access Token (starts with ATCTT3x...): in Bitbucket open Repository settings → Access tokens → Create token with 'Repositories: Read' and 'Pull requests: Read & Write'. Leave 'Bitbucket User' EMPTY and paste the token.",
            "Note: Bitbucket Cloud API does not allow reopening declined PRs — create a new PR from the branch instead.",
            "Click 'Test' in Settings → Git Providers & Tokens to confirm authentication.",
        ],
        command: None,
        link_label: Some("Open Atlassian API Tokens"),
        link_url: Some("https://id.atlassian.com/manage-profile/security/api-tokens"),
        open_providers_settings: true,
    },
    HelpTopic {
        category: "GitLab",
        title: "GitLab Merge Requests return HTTP 401 / 403 / 404",
        symptom: "Missing token, expired PAT, or 'read_api' scope (which allows listing MRs but blocks Approve, Comment, Merge, and Close).",
        steps: &[
            "1. Open GitLab → Preferences → Access Tokens (User Settings → Personal access tokens).",
            "2. Create a token with the 'api' scope (required for approve, merge, comment, close/reopen). Use 'read_api' only for read-only viewing.",
            "3. Paste the 'glpat-...' token in Settings → Git Providers & Tokens → GitLab and click 'Test'.",
            "4. Nested groups (group/subgroup/repo) and self-hosted 'gitlab.*' domains are supported automatically.",
        ],
        command: None,
        link_label: Some("Create GitLab Token"),
        link_url: Some("https://gitlab.com/-/user_settings/personal_access_tokens"),
        open_providers_settings: true,
    },
    HelpTopic {
        category: "Pull Requests",
        title: "Cannot Approve or Merge PR (Own PR, Conflicts, Branch Protection)",
        symptom: "HTTP 405, HTTP 409, HTTP 422, or 'Can not approve your own pull request'.",
        steps: &[
            "1. Own PR: GitHub and Bitbucket forbid approving your own pull request — another reviewer must approve, or click Merge directly if branch rules allow.",
            "2. Merge conflicts: click 'Checkout branch' on the PR, pull/merge or rebase the target branch locally, resolve conflicts, push, then retry Merge.",
            "3. Branch protection: ensure required CI checks are green and minimum reviewer approvals are met.",
            "4. Strategy blocked: some repositories disable 'Rebase' or 'Merge commit' in repo settings — switch the merge dropdown (Merge / Squash / Rebase).",
        ],
        command: None,
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "macOS",
        title: "macOS Gatekeeper: 'gity x.app Not Opened / Cannot Verify Malware'",
        symptom: "macOS flags unsigned open-source apps downloaded from a browser with the com.apple.quarantine attribute.",
        steps: &[
            "1. Copy 'gity x.app' into /Applications.",
            "2. Open Terminal and run the command below to remove the browser quarantine attribute.",
            "3. Or open System Settings → Privacy & Security → scroll down → click 'Open Anyway' next to gity x.",
            "4. Future in-app auto-updates do NOT add quarantine attributes, so you only do this once.",
        ],
        command: Some("xattr -cr \"/Applications/gity x.app\""),
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "macOS",
        title: "macOS Keychain repeatedly asks for 'login' password",
        symptom: "git-credential-osxkeychain prompts repeatedly when background HTTPS git fetches run.",
        steps: &[
            "1. Keep 'Auto-check remote updates on open/refresh' OFF in Settings → General & Graph (off by default).",
            "2. If prompted during Fetch/Pull/Push, enter your macOS login password and click 'Always Allow' once.",
            "3. Best long-term fix: switch your git remote from HTTPS to SSH (uses your SSH key without macOS Keychain HTTPS prompts).",
        ],
        command: Some("git remote -v"),
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "Git",
        title: "Push rejected (non-fast-forward) or Merge/Rebase/Cherry-pick conflicts",
        symptom: "Remote has newer commits, or working tree has conflicting lines during merge/rebase.",
        steps: &[
            "1. Push rejected: click Pull in the top toolbar to integrate remote commits first, then Push again.",
            "2. Conflicts: gity x shows an amber banner at the top with Continue and Abort buttons. Edit conflicted files, stage them, then click Continue (or Abort to cancel safely).",
            "3. Local changes would be overwritten: click Stash (or Commit) before switching branches or pulling, then click Pop afterward.",
        ],
        command: None,
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "AI Reviewer",
        title: "AI PR Review fails (HTTP 401, 404 model not found, 429 quota, or Ollama connection)",
        symptom: "AI Review button reports authentication, billing/quota, invalid model ID, or local server error.",
        steps: &[
            "1. Open Settings → AI Reviewer, pick your provider (OpenAI, Anthropic, OpenRouter, Gemini, or Custom), paste your API key, and click 'Test'.",
            "2. HTTP 401: verify your API key has no billing suspension and matches the selected provider (e.g. sk-... for OpenAI, sk-ant-... for Anthropic, sk-or-... for OpenRouter).",
            "3. HTTP 404 / model not found: select one of the built-in preset models or verify the exact model ID for your provider.",
            "4. HTTP 429: your provider API account has hit a rate limit or has zero credit balance — add credits or switch to OpenRouter/Gemini/local Ollama.",
            "5. Local Ollama: ensure Ollama is running ('ollama serve'), select provider 'Custom', Base URL 'http://localhost:11434/v1/chat/completions', and an installed model name.",
        ],
        command: Some("ollama list"),
        link_label: Some("OpenRouter API Keys"),
        link_url: Some("https://openrouter.ai/keys"),
        open_providers_settings: false,
    },
    HelpTopic {
        category: "Updates",
        title: "Auto-Update did not replace the app automatically",
        symptom: "Update button opens browser or reports permission error.",
        steps: &[
            "1. macOS: make sure 'gity x.app' is inside a writable /Applications or ~/Applications folder (not running directly from inside the mounted .dmg volume!).",
            "2. Linux: automatic in-place updates work with the .AppImage build; .deb installs open the Releases page so you can install with dpkg/apt.",
            "3. Windows: place gity-x.exe in a writable folder (e.g. %LocalAppData%\\Programs\\gity-x) rather than read-only Program Files.",
        ],
        command: None,
        link_label: Some("Open GitHub Releases"),
        link_url: Some("https://github.com/williamjkc69/gity-x/releases/latest"),
        open_providers_settings: false,
    },
];

const TOPICS_ES: &[HelpTopic] = &[
    HelpTopic {
        category: "GitHub",
        title: "PRs de GitHub devuelven HTTP 404 / 401 / 403 (Repos privados, Scopes, SAML SSO)",
        symptom: "GitHub oculta repositorios privados con HTTP 404 cuando no hay token válido, o devuelve HTTP 403 si faltan permisos o autorización SSO.",
        steps: &[
            "1. Recomendado: Inicia sesión una vez con GitHub CLI ('gh auth login') — gity x lo detecta automáticamente en macOS, Linux y Windows.",
            "2. O crea un Personal Access Token clásico con el permiso 'repo' marcado y pégalo en Configuración → Proveedores Git y Tokens → GitHub.",
            "3. Si usas un Fine-grained PAT: selecciona los repositorios objetivo Y activa 'Pull requests: Read and write' + 'Contents: Read'.",
            "4. Repos de organización con SAML SSO: en GitHub → Personal access tokens, pulsa 'Configure SSO' junto a tu token y haz clic en 'Authorize'.",
            "5. Pulsa 'Probar' en Configuración para verificar al instante tu usuario y permisos activos.",
        ],
        command: Some("gh auth login && gh auth status"),
        link_label: Some("Crear Token en GitHub"),
        link_url: Some("https://github.com/settings/tokens/new?scopes=repo&description=gity-x"),
        open_providers_settings: true,
    },
    HelpTopic {
        category: "Bitbucket",
        title: "PRs de Bitbucket devuelven HTTP 404 / 401 / 403 (API Token vs Access Token)",
        symptom: "Bitbucket Cloud devuelve HTTP 404 en repos privados sin credenciales válidas, o 401 si se mezcla autenticación Basic con Bearer.",
        steps: &[
            "Opción A — Atlassian API Token (sirve para todos tus workspaces): crea un API token en id.atlassian.com, pon tu EMAIL de Atlassian en 'Bitbucket User' y el API token en 'Bitbucket Token'.",
            "Opción B — Repository / Workspace Access Token (empieza con ATCTT3x...): en Bitbucket ve a Repository settings → Access tokens → crea token con 'Repositories: Read' y 'Pull requests: Read & Write'. Deja 'Bitbucket User' VACÍO y pega el token.",
            "Nota: La API de Bitbucket Cloud no permite reabrir PRs declinados — debes crear un nuevo PR desde la rama.",
            "Pulsa 'Probar' en Configuración → Proveedores Git y Tokens para validar la conexión.",
        ],
        command: None,
        link_label: Some("Abrir Atlassian API Tokens"),
        link_url: Some("https://id.atlassian.com/manage-profile/security/api-tokens"),
        open_providers_settings: true,
    },
    HelpTopic {
        category: "GitLab",
        title: "Merge Requests de GitLab devuelven HTTP 401 / 403 / 404",
        symptom: "Token faltante, expirado o con permiso 'read_api' (que permite listar MRs pero bloquea Aprobar, Comentar, Merge y Cerrar).",
        steps: &[
            "1. Abre GitLab → Preferencias → Access Tokens (Personal access tokens).",
            "2. Crea un token con permiso 'api' (necesario para aprobar, fusionar, comentar y cerrar/reabrir). Usa 'read_api' solo para lectura.",
            "3. Pega el token 'glpat-...' en Configuración → Proveedores Git y Tokens → GitLab y pulsa 'Probar'.",
            "4. Los grupos anidados (grupo/subgrupo/repo) y servidores propios 'gitlab.*' funcionan automáticamente.",
        ],
        command: None,
        link_label: Some("Crear Token en GitLab"),
        link_url: Some("https://gitlab.com/-/user_settings/personal_access_tokens"),
        open_providers_settings: true,
    },
    HelpTopic {
        category: "Pull Requests",
        title: "No puedo Aprobar o hacer Merge de un PR (PR propio, Conflictos, Protección de rama)",
        symptom: "HTTP 405, HTTP 409, HTTP 422 o 'Can not approve your own pull request'.",
        steps: &[
            "1. PR propio: GitHub y Bitbucket no permiten aprobar tu propio PR — debe aprobarlo otro revisor, o haz Merge directo si las reglas lo permiten.",
            "2. Conflictos de merge: pulsa 'Checkout rama' en el PR, haz merge/rebase con la rama destino localmente, resuelve conflictos, haz Push y reintenta Merge.",
            "3. Protección de rama: verifica que los checks de CI estén en verde y se cumplan las aprobaciones mínimas.",
            "4. Estrategia deshabilitada: algunos repos bloquean 'Rebase' o 'Merge commit' — cambia la estrategia en el selector (Merge / Squash / Rebase).",
        ],
        command: None,
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "macOS",
        title: "macOS Gatekeeper: 'gity x.app no se pudo abrir / no se puede verificar malware'",
        symptom: "macOS marca apps open-source descargadas desde navegador con el atributo com.apple.quarantine.",
        steps: &[
            "1. Arrastra 'gity x.app' a /Applications.",
            "2. Abre Terminal y ejecuta el comando de abajo para quitar el atributo de cuarentena del navegador.",
            "3. O ve a Ajustes del Sistema → Privacidad y seguridad → abajo pulsa 'Abrir de todos modos' junto a gity x.",
            "4. Las futuras auto-actualizaciones dentro de la app NO llevan cuarentena, así que solo se hace una vez.",
        ],
        command: Some("xattr -cr \"/Applications/gity x.app\""),
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "macOS",
        title: "Llavero de macOS (Keychain) pide contraseña repetidamente",
        symptom: "git-credential-osxkeychain muestra alertas cuando corren consultas HTTPS en segundo plano.",
        steps: &[
            "1. Mantén desactivada 'Comprobar actualizaciones remotas al abrir/actualizar' en Configuración → General y Grafo (desactivada por defecto).",
            "2. Si aparece al hacer Fetch/Pull/Push, ingresa tu contraseña de macOS y pulsa 'Permitir siempre' una vez.",
            "3. Mejor solución permanente: cambia el remoto de HTTPS a SSH (usa tu llave SSH sin alertas de Keychain).",
        ],
        command: Some("git remote -v"),
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "Git",
        title: "Push rechazado (non-fast-forward) o conflictos en Merge / Rebase",
        symptom: "El remoto tiene commits más recientes o hay líneas en conflicto en el árbol de trabajo.",
        steps: &[
            "1. Push rechazado: pulsa Pull en la barra superior para traer los cambios remotos primero y vuelve a hacer Push.",
            "2. Conflictos: gity x muestra un banner ámbar arriba con botones Continuar y Abortar. Resuelve los archivos, haz stage y pulsa Continuar (o Abortar para cancelar sin riesgo).",
            "3. Cambios locales se sobrescribirían: usa Stash (o Commit) antes de cambiar de rama o hacer Pull, y luego Pop.",
        ],
        command: None,
        link_label: None,
        link_url: None,
        open_providers_settings: false,
    },
    HelpTopic {
        category: "Revisor IA",
        title: "Revisión de PR con IA falla (HTTP 401, modelo 404, cuota 429 o Ollama local)",
        symptom: "El botón 'Revisar con IA' reporta error de clave API, saldo/cuota, modelo inválido o conexión local.",
        steps: &[
            "1. Abre Configuración → Revisor IA, elige proveedor (OpenAI, Anthropic, OpenRouter, Gemini o Custom), pega tu API key y pulsa 'Probar'.",
            "2. HTTP 401: verifica que la API key corresponda al proveedor seleccionado (sk-... OpenAI, sk-ant-... Anthropic, sk-or-... OpenRouter).",
            "3. HTTP 404 / modelo no encontrado: elige uno de los modelos sugeridos o verifica el ID exacto en tu proveedor.",
            "4. HTTP 429: llegaste al límite de cuota o falta crédito en la cuenta API — recarga créditos o cambia a OpenRouter/Gemini/Ollama local.",
            "5. Ollama local: asegúrate de tener Ollama corriendo ('ollama serve'), selecciona proveedor 'Custom', Base URL 'http://localhost:11434/v1/chat/completions' y un modelo descargado.",
        ],
        command: Some("ollama list"),
        link_label: Some("Abrir OpenRouter Keys"),
        link_url: Some("https://openrouter.ai/keys"),
        open_providers_settings: false,
    },
    HelpTopic {
        category: "Actualizaciones",
        title: "La auto-actualización no reemplazó la app automáticamente",
        symptom: "El botón de actualizar abre el navegador o reporta error de permisos.",
        steps: &[
            "1. macOS: asegúrate de abrir 'gity x.app' desde /Applications o ~/Applications (NO directamente desde el volumen montado del .dmg).",
            "2. Linux: la auto-actualización directa funciona con el .AppImage; si instalaste .deb, abrirá la página de Releases para instalarlo con dpkg/apt.",
            "3. Windows: guarda gity-x.exe en una carpeta con permisos de escritura (ej. %LocalAppData%\\Programs\\gity-x).",
        ],
        command: None,
        link_label: Some("Abrir GitHub Releases"),
        link_url: Some("https://github.com/williamjkc69/gity-x/releases/latest"),
        open_providers_settings: false,
    },
];

pub fn show(ctx: &egui::Context, app: &mut GitKrakenApp) {
    if !app.help_open {
        return;
    }

    let t = app.t();
    let topics = match app.config.language {
        Language::En => TOPICS_EN,
        Language::Es => TOPICS_ES,
    };

    let mut close_now = false;
    let mut open_providers_settings = false;
    let mut copied_cmd: Option<String> = None;

    egui::Window::new(
        RichText::new(format!("?  {}", t.help_center_title))
            .strong()
            .color(Color32::WHITE),
    )
    .collapsible(false)
    .resizable(true)
    .default_width(680.0)
    .default_height(560.0)
    .min_width(480.0)
    .min_height(380.0)
    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
    .show(ctx, |ui| {
        // Search bar + quick Settings button
        ui.horizontal(|ui| {
            let hint = match app.config.language {
                Language::En => "Search error code (404, 401, 403, SAML, Bitbucket, Gatekeeper, Keychain)...",
                Language::Es => "Buscar error (404, 401, 403, SAML, Bitbucket, Gatekeeper, Keychain)...",
            };
            let w = (ui.available_width() - 180.0).max(160.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.help_search)
                    .desired_width(w)
                    .hint_text(hint),
            );
            if !app.help_search.is_empty() && ui.small_button("×").clicked() {
                app.help_search.clear();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(t.settings_tab_providers).clicked() {
                    open_providers_settings = true;
                }
            });
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        let q = app.help_search.trim().to_lowercase();
        let filtered: Vec<(usize, &HelpTopic)> = topics
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if q.is_empty() {
                    return true;
                }
                item.category.to_lowercase().contains(&q)
                    || item.title.to_lowercase().contains(&q)
                    || item.symptom.to_lowercase().contains(&q)
                    || item.steps.iter().any(|s| s.to_lowercase().contains(&q))
            })
            .collect();

        egui::ScrollArea::vertical()
            .id_salt("help_center_scroll")
            .auto_shrink([false, false])
            .max_height(440.0)
            .show(ui, |ui| {
                if filtered.is_empty() {
                    ui.add_space(32.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(match app.config.language {
                                Language::En => "No matching troubleshooting topics.",
                                Language::Es => "No hay temas que coincidan con la búsqueda.",
                            })
                            .color(theme::TEXT_MUTED)
                            .italics(),
                        );
                    });
                    return;
                }

                for (idx, topic) in filtered {
                    egui::Frame::none()
                        .fill(theme::BG_SURFACE)
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(12.0, 9.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let header = RichText::new(format!(
                                "[{}]  {}",
                                topic.category, topic.title
                            ))
                            .size(13.0)
                            .strong()
                            .color(Color32::WHITE);

                            egui::CollapsingHeader::new(header)
                                .id_salt(("help_topic", idx))
                                .default_open(!q.is_empty() || idx < 2)
                                .show(ui, |ui| {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(topic.symptom)
                                            .size(12.0)
                                            .color(theme::ACCENT_AMBER),
                                    );
                                    ui.add_space(6.0);
                                    for step in topic.steps {
                                        ui.label(
                                            RichText::new(*step)
                                                .size(12.0)
                                                .color(theme::TEXT_PRIMARY),
                                        );
                                        ui.add_space(3.0);
                                    }

                                    if let Some(cmd) = topic.command {
                                        ui.add_space(4.0);
                                        egui::Frame::none()
                                            .fill(theme::BG_PRIMARY)
                                            .rounding(Rounding::same(6.0))
                                            .stroke(Stroke::new(1.0_f32, theme::BORDER_ELEVATED))
                                            .inner_margin(Margin::symmetric(10.0, 6.0))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new(cmd)
                                                            .monospace()
                                                            .size(11.5)
                                                            .color(theme::ACCENT_CYAN),
                                                    );
                                                    ui.with_layout(
                                                        Layout::right_to_left(Align::Center),
                                                        |ui| {
                                                            if ui.small_button(t.copy_btn).clicked()
                                                            {
                                                                copied_cmd = Some(cmd.to_string());
                                                            }
                                                        },
                                                    );
                                                });
                                            });
                                    }

                                    if topic.link_url.is_some() || topic.open_providers_settings {
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            if topic.open_providers_settings
                                                && ui.button(t.pr_configure_token).clicked()
                                            {
                                                open_providers_settings = true;
                                            }
                                            if let (Some(lbl), Some(url)) =
                                                (topic.link_label, topic.link_url)
                                            {
                                                if ui
                                                    .button(
                                                        RichText::new(format!("→ {lbl}"))
                                                            .color(theme::ACCENT_CYAN),
                                                    )
                                                    .clicked()
                                                {
                                                    let _ = webbrowser::open(url);
                                                }
                                            }
                                        });
                                    }
                                });
                        });
                    ui.add_space(7.0);
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(t.close).clicked() {
                    close_now = true;
                }
            });
        });
    });

    if let Some(cmd) = copied_cmd {
        ctx.copy_text(cmd);
        app.set_info(t.copied_btn);
    }
    if open_providers_settings {
        app.help_open = false;
        app.settings_tab = SettingsTab::Providers;
        app.settings_open = true;
    }
    if close_now {
        app.help_open = false;
    }
}
