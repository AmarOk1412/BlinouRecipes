/**
 *  Copyright (c) 2022-2023, Sébastien Blin <sebastien.blin@enconn.fr>
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 * this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 * this list of conditions and the following disclaimer in the documentation
 * and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT,
 * INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
 * BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
 * LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE
 * OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/
mod articleparser;
mod config;
mod follow;
mod likes;
mod noteparser;
mod profile;
mod server;

use crate::articleparser::ArticleParser;
use crate::config::Config;
use crate::follow::Followers;
use crate::likes::Likes;
use crate::noteparser::NoteParser;
use crate::profile::Profile;
use crate::server::Server;

use actix_web::{web, web::Data, App, HttpServer};
use std::fs;
use std::sync::Mutex;

// TODO add logs

fn main() {
    // Init logging
    env_logger::init();
    // Run actix_web with tokio to allow both incoming and outgoing requests
    actix_web::rt::System::with_tokio_rt(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(8)
            .thread_name("main-tokio")
            .build()
            .unwrap()
    })
    .block_on(run_server());
}

async fn run_server() {
    let config_str = fs::read_to_string("config.json");
    let config = serde_json::from_str::<Config>(&config_str.unwrap()).unwrap();
    let followers = Followers::new(config.clone());
    let profile = Profile {
        config: config.clone(),
    };
    let likes = Likes::new(config.clone());
    let note_parser = NoteParser::new(config.clone());
    let article_parser = ArticleParser::new(config.clone());
    let server = Data::new(Mutex::new(Server {
        config: config.clone(),
        followers,
        profile,
        likes,
        note_parser,
        article_parser,
    }));
    HttpServer::new(move || {
        App::new()
            .app_data(server.clone())
            .route("/.well-known/webfinger", web::get().to(Server::webfinger))
            .route("/users/chef", web::get().to(Server::profile))
            .route("/users/chef/inbox", web::post().to(Server::inbox))
            .route("/users/chef/outbox", web::get().to(Server::outbox))
            .route(
                "/users/chef/followers",
                web::get().to(Server::user_followers),
            )
            .route(
                "/users/chef/following",
                web::get().to(Server::user_following),
            )
            .route("/users/chef/likes", web::get().to(Server::likes))
    })
    .bind(&*config.bind_address)
    .unwrap()
    .run()
    .await
    .unwrap()
}
