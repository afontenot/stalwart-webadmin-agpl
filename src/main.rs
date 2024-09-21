/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

#![allow(unstable_name_collisions)]
use core::schema::Schemas;
use std::{sync::Arc, time::Duration};

use components::{
    icon::{
        IconAdjustmentsHorizontal, IconChartBarSquare, IconClock, IconDocumentChartBar, IconKey,
        IconLockClosed, IconQueueList, IconShieldCheck, IconSignal, IconSquare2x2, IconUserGroup,
        IconWrench,
    },
    layout::MenuItem,
};

use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pages::{
    account::{
        app_password::{AppPasswordCreate, AppPasswords},
        mfa::ManageMfa,
    },
    config::edit::DEFAULT_SETTINGS_URL,
    manage::spam::{SpamTest, SpamTrain},
};

pub static VERSION_NAME: &str = concat!("Stalwart Management UI v", env!("CARGO_PKG_VERSION"),);

use crate::{
    components::{
        layout::{Layout, LayoutBuilder},
        messages::{alert::init_alerts, modal::init_modals},
    },
    core::oauth::{oauth_refresh_token, AuthToken},
    pages::{
        account::{crypto::ManageCrypto, password::ChangePassword},
        authorize::Authorize,
        config::{edit::SettingsEdit, list::SettingsList, search::SettingsSearch},
        directory::{
            domains::{display::DomainDisplay, edit::DomainCreate, list::DomainList},
            principals::{edit::PrincipalEdit, list::PrincipalList},
        },
        login::Login,
        manage::{logs::Logs, maintenance::Maintenance},
        notfound::NotFound,
        queue::{
            messages::{list::QueueList, manage::QueueManage},
            reports::{display::ReportDisplay, list::ReportList},
        },
        reports::{display::IncomingReportDisplay, list::IncomingReportList},
    },
};

pub mod components;
pub mod core;
pub mod pages;

pub const STATE_STORAGE_KEY: &str = "webadmin_state";
pub const STATE_LOGIN_NAME_KEY: &str = "webadmin_login_name";

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}

#[component]
pub fn App() -> impl IntoView {
    let auth_token = create_rw_signal(
        SessionStorage::get::<AuthToken>(STATE_STORAGE_KEY)
            .map(|mut t| {
                // Force token refresh on reload
                t.is_valid = false;
                t
            })
            .unwrap_or_default(),
    );
    provide_meta_context();
    provide_context(auth_token);
    provide_context(build_schemas());
    init_alerts();
    init_modals();

    // Create a resource to refresh the OAuth token
    let _refresh_token_resource = create_resource(
        move || auth_token.get(),
        move |changed_auth_token| {
            let changed_auth_token = changed_auth_token.clone();

            async move {
                if !changed_auth_token.is_valid && !changed_auth_token.refresh_token.is_empty() {
                    if let Some(grant) = oauth_refresh_token(
                        &changed_auth_token.base_url,
                        &changed_auth_token.refresh_token,
                    )
                    .await
                    {
                        let refresh_token = grant.refresh_token.unwrap_or_default();
                        auth_token.update(|auth_token| {
                            auth_token.access_token = grant.access_token.into();
                            auth_token.refresh_token = refresh_token.clone().into();
                            auth_token.is_valid = true;

                            if let Err(err) =
                                SessionStorage::set(STATE_STORAGE_KEY, auth_token.clone())
                            {
                                log::error!(
                                    "Failed to save authorization token to session storage: {}",
                                    err
                                );
                            }
                        });
                        // Set timer to refresh token
                        if grant.expires_in > 0 && !refresh_token.is_empty() {
                            log::debug!(
                                "Next OAuth token refresh in {} seconds.",
                                grant.expires_in
                            );
                            set_timeout(
                                move || {
                                    auth_token.update(|auth_token| {
                                        auth_token.is_valid = false;
                                    });
                                },
                                Duration::from_secs(grant.expires_in),
                            );
                        }
                    }
                }
            }
        },
    );

    let is_logged_in = create_memo(move |_| auth_token.get().is_logged_in());
    let is_admin = create_memo(move |_| auth_token.get().is_admin());

    view! {
        <Router>
            <Routes>
                <ProtectedRoute
                    path="/manage"
                    view=move || {
                        view! { <Layout menu_items=LayoutBuilder::manage() is_admin=is_admin/> }
                    }

                    redirect_path="/login"
                    condition=move || is_logged_in.get()
                >
                    <ProtectedRoute
                        path="/directory/domains"
                        view=DomainList
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/directory/domains/edit"
                        view=DomainCreate
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/directory/domains/:id/view"
                        view=DomainDisplay
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />

                    <ProtectedRoute
                        path="/directory/:object"
                        view=PrincipalList
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/directory/:object/:id?/edit"
                        view=PrincipalEdit
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/queue/messages"
                        view=QueueList
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/queue/message/:id"
                        view=QueueManage
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/queue/reports"
                        view=ReportList
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/queue/report/:id"
                        view=ReportDisplay
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/reports/:object"
                        view=IncomingReportList
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/reports/:object/:id"
                        view=IncomingReportDisplay
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/logs"
                        view=Logs
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/spam/train"
                        view=SpamTrain
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/spam/test"
                        view=SpamTest
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/maintenance"
                        view=Maintenance
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                </ProtectedRoute>
                <ProtectedRoute
                    path="/settings"
                    view=move || {
                        view! { <Layout menu_items=LayoutBuilder::settings() is_admin=is_admin/> }
                    }

                    redirect_path="/login"
                    condition=move || is_admin.get()
                >
                    <ProtectedRoute
                        path="/:object"
                        view=SettingsList
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/:object/:id?/edit"
                        view=SettingsEdit
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                    <ProtectedRoute
                        path="/search"
                        view=SettingsSearch
                        redirect_path="/login"
                        condition=move || is_admin.get()
                    />
                </ProtectedRoute>
                <ProtectedRoute
                    path="/account"
                    view=move || {
                        view! { <Layout menu_items=LayoutBuilder::account() is_admin=is_admin/> }
                    }

                    redirect_path="/login"
                    condition=move || is_logged_in.get()
                >
                    <ProtectedRoute
                        path="/crypto"
                        view=ManageCrypto
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/password"
                        view=ChangePassword
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/mfa"
                        view=ManageMfa
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/app-passwords"
                        view=AppPasswords
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />
                    <ProtectedRoute
                        path="/app-passwords/edit"
                        view=AppPasswordCreate
                        redirect_path="/login"
                        condition=move || is_logged_in.get()
                    />

                </ProtectedRoute>

                <Route path="/" view=Login/>
                <Route path="/login" view=Login/>
                <Route path="/authorize/:type?" view=Authorize/>
                <Route path="/*any" view=NotFound/>
            </Routes>
        </Router>
        <div id="portal_root"></div>
    }
}

impl LayoutBuilder {
    pub fn manage() -> Vec<MenuItem> {
        LayoutBuilder::new("/manage")
            .create("Dashboard")
            .icon(view! { <IconChartBarSquare/> })
            .create("Overview")
            .route("/dashboard/overview")
            .insert()
            .create("Network")
            .route("/dashboard/network")
            .insert()
            .create("Security")
            .route("/dashboard/security")
            .insert()
            .create("Delivery")
            .route("/dashboard/delivery")
            .insert()
            .create("Performance")
            .route("/dashboard/performance")
            .insert()
            .insert()
            .create("Directory")
            .icon(view! { <IconUserGroup/> })
            .create("Accounts")
            .route("/directory/accounts")
            .insert()
            .create("Groups")
            .route("/directory/groups")
            .insert()
            .create("Lists")
            .route("/directory/lists")
            .insert()
            .create("Domains")
            .route("/directory/domains")
            .insert()
            .insert()
            .create("Queues")
            .icon(view! { <IconQueueList/> })
            .create("Messages")
            .route("/queue/messages")
            .insert()
            .create("Reports")
            .route("/queue/reports")
            .insert()
            .insert()
            .create("Reports")
            .icon(view! { <IconDocumentChartBar/> })
            .create("DMARC Aggregate")
            .route("/reports/dmarc")
            .insert()
            .create("TLS Aggregate")
            .route("/reports/tls")
            .insert()
            .create("Failures")
            .route("/reports/arf")
            .insert()
            .insert()
            .create("History")
            .icon(view! { <IconClock/> })
            .create("Received Messages")
            .route("/tracing/received")
            .insert()
            .create("Delivery Attempts")
            .route("/tracing/delivery")
            .insert()
            .insert()
            .create("Telemetry")
            .icon(view! { <IconSignal/> })
            .create("Logs")
            .route("/logs")
            .insert()
            .create("Live tracing")
            .route("/tracing/live")
            .insert()
            .insert()
            .create("Antispam")
            .icon(view! { <IconShieldCheck/> })
            .create("Train")
            .route("/spam/train")
            .insert()
            .create("Test")
            .route("/spam/test")
            .insert()
            .insert()
            .create("Settings")
            .icon(view! { <IconAdjustmentsHorizontal/> })
            .raw_route(DEFAULT_SETTINGS_URL)
            .insert()
            .create("Maintenance")
            .icon(view! { <IconWrench/> })
            .route("/maintenance")
            .insert()
            .menu_items
    }

    pub fn account() -> Vec<MenuItem> {
        LayoutBuilder::new("/account")
            .create("Encryption-at-rest")
            .icon(view! { <IconLockClosed/> })
            .route("/crypto")
            .insert()
            .create("Change Password")
            .icon(view! { <IconKey/> })
            .route("/password")
            .insert()
            .create("Two-factor Auth")
            .icon(view! { <IconShieldCheck/> })
            .route("/mfa")
            .insert()
            .create("App Passwords")
            .icon(view! { <IconSquare2x2/> })
            .route("/app-passwords")
            .insert()
            .menu_items
    }
}

pub fn build_schemas() -> Arc<Schemas> {
    Schemas::builder()
        .build_login()
        .build_principals()
        .build_domains()
        .build_store()
        .build_directory()
        .build_authentication()
        .build_storage()
        .build_tls()
        .build_server()
        .build_listener()
        .build_telemetry()
        .build_smtp_inbound()
        .build_smtp_outbound()
        .build_mail_auth()
        .build_jmap()
        .build_imap()
        .build_sieve()
        .build_spam_lists()
        .build_spam_manage()
        .build_password_change()
        .build_crypto()
        .build_authorize()
        .build_mfa()
        .build_app_passwords()
        .build()
        .into()
}
