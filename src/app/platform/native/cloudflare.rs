//! Cloudflare DNS API integration for adding a multiaddr to a TXT record.
//!
//! Creates a separate TXT record for up to [MAX_RECORDS] multiaddrs, then overwrites older records with newer ones.
//!
//! Cloudflare [cloudflare::endpoints::dns::DnsRecord] provides a unique descriptions for specific records, so we use
//! comments to insert a timestamp for each record. This is used to determine which records to keep
//! and which to remove.
use cloudflare::endpoints::dns::{
    CreateDnsRecord, DeleteDnsRecord, DnsContent, ListDnsRecords, ListDnsRecordsParams,
};
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{async_api::Client, auth::Credentials, Environment};
use cloudflare::framework::{HttpApiClientConfig, OrderDirection};
use multiaddr::Multiaddr;

/// Cloudflare DNS Errors
#[derive(Debug, thiserror::Error)]
pub enum CloudflareError {
    /// Cloudflare API Error
    #[error("Cloudflare API Error: {0}")]
    ApiError(#[from] cloudflare::framework::Error),
    /// API Failure
    #[error("API Failure: {0}")]
    ApiFailure(#[from] ApiFailure),
}

/// Add a multiaddr to a TXT record in Cloudflare. removes older records if there are more than [MAX_RECORDS]
#[bon::builder]
pub async fn add_address(
    api_token: String,
    zone_id: String,
    txt_name: String,
    multiaddr: Multiaddr,
) -> Result<Vec<String>, CloudflareError> {
    let mut log = vec![];

    /// The maximum number of TXT records to keep in Cloudflare
    const MAX_RECORDS: usize = 2;

    let credentials = Credentials::UserAuthToken { token: api_token };
    let api_client = Client::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .map_err(|e| {
        tracing::error!("❌ Error creating Cloudflare API client: {:?}", e);
        CloudflareError::ApiError(e)
    })?;

    let mut existing_records = api_client
        .request(&ListDnsRecords {
            zone_identifier: &zone_id,
            params: ListDnsRecordsParams {
                direction: Some(OrderDirection::Descending),
                name: Some(txt_name.clone()),
                page: Some(1),
                per_page: Some(100),
                ..Default::default()
            },
        })
        .await
        .map_err(|e| {
            tracing::error!("❌ Error listing DNS records: {:?}", e);
            CloudflareError::ApiFailure(e)
        })?
        .result;

    existing_records.sort_by(|a, b| a.created_on.cmp(&b.created_on).reverse());

    // check to see if the multiaddr is already in an existing record, if so, return
    if existing_records.iter().any(|record| {
        if let DnsContent::TXT { content } = &record.content {
            content.contains(&multiaddr.to_string())
        } else {
            false
        }
    }) {
        let msg = format!("🔍 Multiaddr already exists in TXT record: {:?}", multiaddr);
        tracing::info!("{}", msg);
        log.push(msg);
        return Ok(log);
    }

    // Create new record
    // turn the addr into a dnsaddr
    let content = format!("dnsaddr={}", multiaddr);
    api_client
        .request(&CreateDnsRecord {
            zone_identifier: &zone_id,
            params: cloudflare::endpoints::dns::CreateDnsRecordParams {
                ttl: None,
                priority: None,
                proxied: None,
                name: &txt_name,
                content: DnsContent::TXT {
                    content: content.to_string(),
                },
            },
        })
        .await?;
    let msg = format!("🆕 TXT record created successfully {:?}", content);
    tracing::info!("{}", msg);
    log.push(msg);

    // if num <= MAX_RECORDS, return
    if existing_records.len() <= MAX_RECORDS {
        let msg = format!(
            "📦 TXT records within limit (found {})",
            existing_records.len()
        );
        tracing::info!("{}", msg);
        log.push(msg);
        return Ok(log);
    }

    let msg = format!(
        "🗑 Deleting old TXT records ({} > {})",
        existing_records.len(),
        MAX_RECORDS
    );
    tracing::info!("{}", msg);

    // Delete all but most recent MAX_RECORDS
    for record in existing_records.iter().skip(MAX_RECORDS) {
        api_client
            .request(&DeleteDnsRecord {
                zone_identifier: &zone_id,
                identifier: &record.id,
            })
            .await?;
        let msg = format!(
            "🗑 Deleted old TXT record [{}]: {:?}",
            record.created_on, record.id
        );
        tracing::info!("{}", msg);
        log.push(msg);
    }

    Ok(log)
}

#[cfg(test)]
mod tests {
    // use super::*;

    use multiaddr::{Multiaddr, Protocol};
    use std::net::Ipv6Addr;

    #[tokio::test]
    async fn test_add_address() {
        // Create a multiaddr

        let address_webrtc = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
            .with(Protocol::Udp(0))
            .with(Protocol::WebRTCDirect);

        assert_eq!(
            address_webrtc.to_string(),
            "/ip6/::/udp/0/webrtc-direct".to_string()
        );

        //let r = add_address()
        //    .api_token("api_token".to_string())
        //    .zone_id("zone_id".to_string())
        //    .txt_name("txt_name".to_string())
        //    .multiaddr(&address_webrtc)
        //    .call()
        //    .await;
    }
}
