use crate::{store, AppConfig};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use tempfile::TempDir;
use zksync_concurrency::ctx;
use zksync_consensus_roles::{node, validator::testonly::Setup};
use zksync_consensus_storage::{testonly, PersistentBlockStore};
use zksync_protobuf::testonly::test_encode_random;

fn make_addr<R: Rng + ?Sized>(rng: &mut R) -> std::net::SocketAddr {
    std::net::SocketAddr::new(std::net::IpAddr::from(rng.gen::<[u8; 16]>()), rng.gen())
}

impl Distribution<AppConfig> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AppConfig {
        AppConfig {
            server_addr: make_addr(rng),
            public_addr: make_addr(rng),
            metrics_server_addr: Some(make_addr(rng)),

            genesis: rng.gen(),

            gossip_dynamic_inbound_limit: rng.gen(),
            gossip_static_inbound: (0..5)
                .map(|_| rng.gen::<node::SecretKey>().public())
                .collect(),
            gossip_static_outbound: (0..6)
                .map(|_| (rng.gen::<node::SecretKey>().public(), make_addr(rng)))
                .collect(),
            max_payload_size: rng.gen(),
        }
    }
}

#[test]
fn test_schema_encoding() {
    let ctx = ctx::test_root(&ctx::RealClock);
    let rng = &mut ctx.rng();
    test_encode_random::<AppConfig>(rng);
}

#[tokio::test]
async fn test_reopen_rocksdb() {
    let ctx = &ctx::test_root(&ctx::RealClock);
    let rng = &mut ctx.rng();
    let dir = TempDir::new().unwrap();
    let mut setup = Setup::new(rng, 3);
    setup.push_blocks(rng, 5);
    let mut want = vec![];
    for b in &setup.blocks {
        let store = store::RocksDB::open(setup.genesis.clone(), dir.path())
            .await
            .unwrap();
        store.store_next_block(ctx, b).await.unwrap();
        want.push(b.clone());
        assert_eq!(want, testonly::dump(ctx, &store).await);
    }
}
