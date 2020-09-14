use std::collections::{HashMap, HashSet};
use std::io::Write;

use anyhow::Result;
use colored::Colorize;
use join_lazy_fmt::Join;
use proto::api::inbox::{ArchiveRequest, GetResponse, ListResponse};

use crate::client::Client;
use crate::output::{Output, SimpleOutput};

pub async fn exec(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    return match matches.subcommand() {
        ("list", Some(matches)) => list(matches, client).await,
        ("show", Some(matches)) => show(matches, client).await,
        ("delete", Some(matches)) => delete(matches, client).await,
        ("archive", Some(matches)) => archive(matches, client).await,

        _ => unreachable!()
    };
}


pub async fn list(_: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let response = client.inbox_list().await?;
    return Ok(Box::new(response));
}

pub async fn show(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    let response = client.inbox_get(id).await?;
    return Ok(Box::new(response));
}

pub async fn delete(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    let response = client.inbox_delete(id).await?;
    return Ok(Box::new(response));
}

pub async fn archive(matches: &clap::ArgMatches<'_>, client: &mut Client) -> Result<Box<dyn Output>> {
    let id = matches.value_of("id").expect("Required ID missing");

    let labels = matches.values_of("labels")
        .map(|labels| labels.map(str::to_string).collect())
        .unwrap_or_else(HashSet::default);

    // TODO: Custom typed parsers in clap?
    let properties = matches.values_of("labels")
        .map(|properties| properties.map(|property| property.split_once('=').unwrap_or((property, ""))).map(|(k, v)| (k.to_string(), v.to_string())).collect())
        .unwrap_or_else(HashMap::default);

    let data = ArchiveRequest {
        labels,
        properties,
    };

    let response = client.inbox_archive(id, &data).await?;
    return Ok(Box::new(response));
}

impl SimpleOutput for ListResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        if self.count == 0 {
            writeln!(w, "{}", "Inbox is empty".bright_green())?;
        } else {
            writeln!(w, "{} documents in inbox", self.count.to_string().bright_yellow())?;

            for doc in &self.docs {
                writeln!(w, "  {}", doc.bright_cyan())?;
            }
        }

        return Ok(());
    }
}

impl SimpleOutput for GetResponse {
    fn to_text(&self, w: &mut dyn Write) -> Result<()> {
        writeln!(w, "Document {}:", self.id.bright_cyan())?;
        writeln!(w, "  Uploaded: {}", self.uploaded)?;

        if !self.labels.is_empty() {
            writeln!(w, "  Labels: {}", ", ".join(self.labels.iter().map(|s| s.bright_purple())))?;
        }

        if !self.properties.is_empty() {
            writeln!(w, "  Properties: ")?;
            for (key, val) in &self.properties {
                writeln!(w, "  {}: {}", key, val.bright_blue())?;
            }
        }

        return Ok(());
    }
}
