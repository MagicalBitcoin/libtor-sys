extern crate autotools;
extern crate cc;
extern crate fs_extra;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fs_extra::dir::{copy, CopyOptions};

pub fn source_dir(var: &str, package: &str, version: &str) -> PathBuf {
    Path::new(var).join(format!("{}-{}", package, version))
}

pub fn get_version(full_version: &str) -> String {
    let parts: Vec<_> = full_version.split('+').collect();
    parts[1].into()
}

pub struct Artifacts {
    pub root: PathBuf,
    pub include_dir: PathBuf,
    pub lib_dir: PathBuf,
    pub libs: Vec<String>,
}

impl Artifacts {
    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}

pub fn autoreconf(path: &PathBuf) -> Result<(), Vec<u8>> {
    match Command::new("autoreconf")
        .current_dir(path)
        .args(&["--force", "--install"])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                Err(output.stderr)
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("{:?}", e).as_bytes().to_vec()),
    }
}

fn build_libevent() -> Artifacts {
    // TODO: cmake on windows
    const LIBEVENT_VERSION: &'static str = "2.1.11-stable";

    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    let mut cc = cc::Build::new();
    cc.target(&target).host(&host);
    let compiler = cc.get_compiler();
    let root = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR expected")).join("libevent");
    fs::create_dir_all(&root).expect("Cannot create `libevent` in `OUT_DIR`");

    let original_src = source_dir(env!("CARGO_MANIFEST_DIR"), "libevent", LIBEVENT_VERSION);
    let path = source_dir(root.to_str().unwrap(), "libevent", LIBEVENT_VERSION);
    if !path.exists() {
        copy(original_src, &root, &CopyOptions::new())
            .expect("Unable to copy libevent's src folder");
    }

    if let Err(e) = autoreconf(&path) {
        println!(
            "cargo:warning=Failed to run `autoreconf`: {:?}",
            String::from_utf8(e)
        );
    }

    let mut config = autotools::Config::new(path.clone());
    config
        .out_dir(&root)
        .env("CC", compiler.path())
        .host(&target)
        .enable_static()
        .disable_shared()
        .with("pic", None)
        .disable("samples", None)
        .disable("openssl", None)
        .disable("libevent-regress", None)
        .disable("debug-mode", None)
        .disable("dependency-tracking", None);

    let libevent = config.build();
    let artifacts = Artifacts {
        lib_dir: libevent.join("lib"),
        include_dir: root.join("include"),
        libs: vec!["event".to_string(), "event_pthreads".to_string()], // TODO: on windows re-add the `lib` prefix
        root,
    };
    artifacts.print_cargo_metadata();

    artifacts
}

fn build_tor(libevent: Artifacts) {
    let target = env::var("TARGET").expect("TARGET expected");

    // TODO https://github.com/arlolra/tor/blob/master/INSTALL#L32
    let openssl_dir =
        PathBuf::from(env::var("DEP_OPENSSL_ROOT").expect("DEP_OPENSSL_ROOT expected"));

    let out_dir = format!("\"{}\"", env::var("OUT_DIR").unwrap());
    let full_version = env!("CARGO_PKG_VERSION");
    let original_src = source_dir(
        env!("CARGO_MANIFEST_DIR"),
        "tor-tor",
        &get_version(full_version),
    );

    let mut build = cc::Build::new();
    build.define("HAVE_CONFIG_H", None);
    build.define("BINDIR", Some(out_dir.as_str()));
    build.define("SHARE_DATADIR", Some(out_dir.as_str()));
    build.define("LOCALSTATEDIR", Some(out_dir.as_str()));

    build.include(PathBuf::from("configs").join(&target));

    build.include(&original_src);
    build.include(original_src.join("src"));
    build.include(original_src.join("src/ext"));
    build.include(original_src.join("src/ext/trunnel"));
    build.include(original_src.join("src/trunnel"));

    build.include(openssl_dir.join("include/"));
    build.include(libevent.include_dir);

    for file in FILE_LIST.iter() {
        let full_path = original_src.join(file);
        build.file(full_path);
    }

    if target.contains("android") {
        // Apparently zlib is already there on Android https://github.com/rust-lang/libz-sys/blob/master/build.rs#L42

        let sysroot_lib = format!("{}/usr/lib", env::var("SYSROOT").expect("SYSROOT expected"));
        println!("cargo:rustc-link-search=native={}", sysroot_lib);
    } else {
        let mut zlib_dir = PathBuf::from(env::var("DEP_Z_ROOT").expect("DEP_Z_ROOT expected"));
        let zlib_include_dir = zlib_dir.join("include");
        zlib_dir.push("build");

        build.include(zlib_include_dir);

        println!("cargo:rustc-link-search=native={}", zlib_dir.display());
    }

    build.compile("libtor.a");

    println!("cargo:rustc-link-lib=static={}", "event");
    println!("cargo:rustc-link-lib=static={}", "event_pthreads");

    println!("cargo:rustc-link-lib=static={}", "crypto");
    println!("cargo:rustc-link-lib=static={}", "ssl");

    println!("cargo:rustc-link-lib=static={}", "z");

    println!("cargo:rustc-link-lib=static={}", "tor");

    println!(
        "cargo:include={}",
        original_src.join("src/feature/api/").display()
    );

    // TODO: remove
    println!("cargo:rerun-if-changed=build.rs");
}

fn main() {
    let libevent = build_libevent();
    build_tor(libevent);
}

const FILE_LIST: [&str; 348] = [
    "src/ext/csiphash.c",
    //"src/tools/tor_runner.c",
    "src/app/config/config.c",
    //"src/app/main/tor_main.c",
    "src/app/config/statefile.c",
    "src/app/main/main.c",
    "src/app/main/shutdown.c",
    "src/app/main/subsystem_list.c",
    "src/core/crypto/hs_ntor.c",
    "src/app/main/subsysmgr.c",
    "src/core/crypto/onion_crypto.c",
    "src/core/crypto/onion_fast.c",
    "src/core/crypto/onion_ntor.c",
    "src/core/crypto/onion_tap.c",
    "src/core/crypto/relay_crypto.c",
    "src/core/mainloop/connection.c",
    "src/core/mainloop/cpuworker.c",
    "src/core/mainloop/mainloop.c",
    "src/core/mainloop/mainloop_pubsub.c",
    "src/core/mainloop/mainloop_sys.c",
    "src/core/mainloop/netstatus.c",
    "src/core/mainloop/periodic.c",
    "src/core/or/address_set.c",
    "src/core/or/channel.c",
    "src/core/or/channelpadding.c",
    "src/core/or/channeltls.c",
    "src/core/or/circuitlist.c",
    "src/core/or/circuitbuild.c",
    "src/core/or/circuitmux.c",
    "src/core/or/circuitmux_ewma.c",
    "src/core/or/circuitpadding.c",
    "src/core/or/circuitpadding_machines.c",
    "src/core/or/circuitstats.c",
    "src/core/or/circuituse.c",
    "src/core/or/crypt_path.c",
    "src/core/or/command.c",
    "src/core/or/connection_edge.c",
    "src/core/or/connection_or.c",
    "src/core/or/dos.c",
    "src/core/or/onion.c",
    "src/core/or/ocirc_event.c",
    "src/core/or/or_periodic.c",
    "src/core/or/or_sys.c",
    "src/core/or/orconn_event.c",
    "src/core/or/policies.c",
    "src/core/or/protover.c",
    "src/core/or/protover_rust.c",
    "src/core/or/reasons.c",
    "src/core/or/relay.c",
    "src/core/or/scheduler.c",
    "src/core/or/scheduler_kist.c",
    "src/core/or/scheduler_vanilla.c",
    "src/core/or/sendme.c",
    "src/core/or/status.c",
    "src/core/or/versions.c",
    "src/core/proto/proto_cell.c",
    "src/core/proto/proto_control0.c",
    "src/core/proto/proto_ext_or.c",
    "src/core/proto/proto_http.c",
    "src/core/proto/proto_socks.c",
    "src/feature/api/tor_api.c",
    "src/feature/client/addressmap.c",
    "src/feature/client/bridges.c",
    "src/feature/client/circpathbias.c",
    "src/feature/client/dnsserv.c",
    "src/feature/client/entrynodes.c",
    "src/feature/client/transports.c",
    "src/feature/control/btrack.c",
    "src/feature/control/btrack_circuit.c",
    "src/feature/control/btrack_orconn.c",
    "src/feature/control/btrack_orconn_cevent.c",
    "src/feature/control/btrack_orconn_maps.c",
    "src/feature/control/control.c",
    "src/feature/control/control_auth.c",
    "src/feature/control/control_bootstrap.c",
    "src/feature/control/control_cmd.c",
    "src/feature/control/control_events.c",
    "src/feature/control/control_fmt.c",
    "src/feature/control/control_getinfo.c",
    "src/feature/control/control_proto.c",
    "src/feature/control/fmt_serverstatus.c",
    "src/feature/control/getinfo_geoip.c",
    "src/feature/dircache/conscache.c",
    "src/feature/dircache/consdiffmgr.c",
    "src/feature/dircache/dircache.c",
    "src/feature/dircache/dirserv.c",
    "src/feature/dirclient/dirclient.c",
    "src/feature/dirclient/dlstatus.c",
    "src/feature/dircommon/consdiff.c",
    "src/feature/dircommon/directory.c",
    "src/feature/dircommon/fp_pair.c",
    "src/feature/dircommon/voting_schedule.c",
    "src/feature/dirparse/authcert_parse.c",
    "src/feature/dirparse/microdesc_parse.c",
    "src/feature/dirparse/ns_parse.c",
    "src/feature/dirparse/parsecommon.c",
    "src/feature/dirparse/policy_parse.c",
    "src/feature/dirparse/routerparse.c",
    "src/feature/dirparse/sigcommon.c",
    "src/feature/dirparse/signing.c",
    "src/feature/dirparse/unparseable.c",
    "src/feature/hibernate/hibernate.c",
    "src/feature/hs/hs_cache.c",
    "src/feature/hs/hs_cell.c",
    "src/feature/hs/hs_circuit.c",
    "src/feature/hs/hs_circuitmap.c",
    "src/feature/hs/hs_client.c",
    "src/feature/hs/hs_common.c",
    "src/feature/hs/hs_config.c",
    "src/feature/hs/hs_control.c",
    "src/feature/hs/hs_descriptor.c",
    "src/feature/hs/hs_dos.c",
    "src/feature/hs/hs_ident.c",
    "src/feature/hs/hs_intropoint.c",
    "src/feature/hs/hs_service.c",
    "src/feature/hs/hs_stats.c",
    "src/feature/hs_common/replaycache.c",
    "src/feature/hs_common/shared_random_client.c",
    "src/feature/keymgt/loadkey.c",
    "src/feature/nodelist/authcert.c",
    "src/feature/nodelist/describe.c",
    "src/feature/nodelist/dirlist.c",
    "src/feature/nodelist/microdesc.c",
    "src/feature/nodelist/networkstatus.c",
    "src/feature/nodelist/nickname.c",
    "src/feature/nodelist/nodefamily.c",
    "src/feature/nodelist/nodelist.c",
    "src/feature/nodelist/node_select.c",
    "src/feature/nodelist/routerinfo.c",
    "src/feature/nodelist/routerlist.c",
    "src/feature/nodelist/routerset.c",
    "src/feature/nodelist/fmt_routerstatus.c",
    "src/feature/nodelist/torcert.c",
    "src/feature/relay/dns.c",
    "src/feature/relay/ext_orport.c",
    "src/feature/relay/onion_queue.c",
    "src/feature/relay/relay_periodic.c",
    "src/feature/relay/relay_sys.c",
    "src/feature/relay/router.c",
    "src/feature/relay/routerkeys.c",
    "src/feature/relay/routermode.c",
    "src/feature/relay/selftest.c",
    "src/feature/rend/rendcache.c",
    "src/feature/rend/rendclient.c",
    "src/feature/rend/rendcommon.c",
    "src/feature/rend/rendmid.c",
    "src/feature/rend/rendparse.c",
    "src/feature/rend/rendservice.c",
    "src/feature/stats/geoip_stats.c",
    "src/feature/stats/rephist.c",
    "src/feature/stats/predict_ports.c",
    "src/lib/compress/compress.c",
    "src/lib/compress/compress_buf.c",
    "src/lib/compress/compress_lzma.c",
    "src/lib/compress/compress_none.c",
    "src/lib/compress/compress_zlib.c",
    "src/lib/compress/compress_zstd.c",
    "src/lib/evloop/compat_libevent.c",
    "src/lib/evloop/evloop_sys.c",
    "src/lib/evloop/procmon.c",
    "src/lib/evloop/timers.c",
    "src/lib/evloop/token_bucket.c",
    "src/lib/evloop/workqueue.c",
    "src/lib/tls/buffers_tls.c",
    "src/lib/tls/tortls.c",
    "src/lib/tls/x509.c",
    "src/lib/tls/tortls_openssl.c",
    "src/lib/tls/x509_openssl.c",
    "src/lib/crypt_ops/crypto_cipher.c",
    "src/lib/crypt_ops/crypto_curve25519.c",
    "src/lib/crypt_ops/crypto_dh.c",
    "src/lib/crypt_ops/crypto_digest.c",
    "src/lib/crypt_ops/crypto_ed25519.c",
    "src/lib/crypt_ops/crypto_format.c",
    "src/lib/crypt_ops/crypto_hkdf.c",
    "src/lib/crypt_ops/crypto_init.c",
    "src/lib/crypt_ops/crypto_ope.c",
    "src/lib/crypt_ops/crypto_pwbox.c",
    "src/lib/crypt_ops/crypto_rand.c",
    "src/lib/crypt_ops/crypto_rand_fast.c",
    "src/lib/crypt_ops/crypto_rand_numeric.c",
    "src/lib/crypt_ops/crypto_rsa.c",
    "src/lib/crypt_ops/crypto_s2k.c",
    "src/lib/crypt_ops/crypto_util.c",
    "src/lib/crypt_ops/digestset.c",
    "src/lib/crypt_ops/aes_openssl.c",
    "src/lib/crypt_ops/crypto_digest_openssl.c",
    "src/lib/crypt_ops/crypto_rsa_openssl.c",
    "src/lib/crypt_ops/crypto_dh_openssl.c",
    "src/lib/crypt_ops/crypto_openssl_mgt.c",
    "src/ext/keccak-tiny/keccak-tiny-unrolled.c",
    "src/ext/curve25519_donna/curve25519-donna-c64.c",
    "src/ext/ed25519/ref10/fe_0.c",
    "src/ext/ed25519/ref10/fe_1.c",
    "src/ext/ed25519/ref10/fe_add.c",
    "src/ext/ed25519/ref10/fe_cmov.c",
    "src/ext/ed25519/ref10/fe_copy.c",
    "src/ext/ed25519/ref10/fe_frombytes.c",
    "src/ext/ed25519/ref10/fe_invert.c",
    "src/ext/ed25519/ref10/fe_isnegative.c",
    "src/ext/ed25519/ref10/fe_isnonzero.c",
    "src/ext/ed25519/ref10/fe_mul.c",
    "src/ext/ed25519/ref10/fe_neg.c",
    "src/ext/ed25519/ref10/fe_pow22523.c",
    "src/ext/ed25519/ref10/fe_sq.c",
    "src/ext/ed25519/ref10/fe_sq2.c",
    "src/ext/ed25519/ref10/fe_sub.c",
    "src/ext/ed25519/ref10/fe_tobytes.c",
    "src/ext/ed25519/ref10/ge_add.c",
    "src/ext/ed25519/ref10/ge_double_scalarmult.c",
    "src/ext/ed25519/ref10/ge_frombytes.c",
    "src/ext/ed25519/ref10/ge_madd.c",
    "src/ext/ed25519/ref10/ge_msub.c",
    "src/ext/ed25519/ref10/ge_p1p1_to_p2.c",
    "src/ext/ed25519/ref10/ge_p1p1_to_p3.c",
    "src/ext/ed25519/ref10/ge_p2_0.c",
    "src/ext/ed25519/ref10/ge_p2_dbl.c",
    "src/ext/ed25519/ref10/ge_p3_0.c",
    "src/ext/ed25519/ref10/ge_p3_dbl.c",
    "src/ext/ed25519/ref10/ge_p3_to_cached.c",
    "src/ext/ed25519/ref10/ge_p3_to_p2.c",
    "src/ext/ed25519/ref10/ge_p3_tobytes.c",
    "src/ext/ed25519/ref10/ge_precomp_0.c",
    "src/ext/ed25519/ref10/ge_scalarmult_base.c",
    "src/ext/ed25519/ref10/ge_sub.c",
    "src/ext/ed25519/ref10/ge_tobytes.c",
    "src/ext/ed25519/ref10/keypair.c",
    "src/ext/ed25519/ref10/open.c",
    "src/ext/ed25519/ref10/sc_muladd.c",
    "src/ext/ed25519/ref10/sc_reduce.c",
    "src/ext/ed25519/ref10/sign.c",
    "src/ext/ed25519/ref10/keyconv.c",
    "src/ext/ed25519/ref10/blinding.c",
    "src/ext/ed25519/donna/ed25519_tor.c",
    "src/lib/geoip/geoip.c",
    "src/lib/process/daemon.c",
    "src/lib/process/env.c",
    "src/lib/process/pidfile.c",
    "src/lib/process/process.c",
    "src/lib/process/process_sys.c",
    "src/lib/process/process_unix.c",
    "src/lib/process/process_win32.c",
    "src/lib/process/restrict.c",
    "src/lib/process/setuid.c",
    "src/lib/process/waitpid.c",
    "src/lib/process/winprocess_sys.c",
    "src/lib/buf/buffers.c",
    "src/lib/confmgt/confparse.c",
    "src/lib/confmgt/structvar.c",
    "src/lib/confmgt/type_defs.c",
    "src/lib/confmgt/typedvar.c",
    "src/lib/confmgt/unitparse.c",
    "src/lib/pubsub/pubsub_build.c",
    "src/lib/pubsub/pubsub_check.c",
    "src/lib/pubsub/pubsub_publish.c",
    "src/lib/dispatch/dispatch_cfg.c",
    "src/lib/dispatch/dispatch_core.c",
    "src/lib/dispatch/dispatch_naming.c",
    "src/lib/dispatch/dispatch_new.c",
    "src/lib/time/compat_time.c",
    "src/lib/time/time_sys.c",
    "src/lib/time/tvdiff.c",
    "src/lib/fs/conffile.c",
    "src/lib/fs/dir.c",
    "src/lib/fs/files.c",
    "src/lib/fs/freespace.c",
    "src/lib/fs/lockfile.c",
    "src/lib/fs/mmap.c",
    "src/lib/fs/path.c",
    "src/lib/fs/storagedir.c",
    "src/lib/fs/userdb.c",
    "src/lib/encoding/binascii.c",
    "src/lib/encoding/confline.c",
    "src/lib/encoding/cstring.c",
    "src/lib/encoding/keyval.c",
    "src/lib/encoding/kvline.c",
    "src/lib/encoding/pem.c",
    "src/lib/encoding/qstring.c",
    "src/lib/encoding/time_fmt.c",
    "src/lib/sandbox/sandbox.c",
    "src/lib/container/bloomfilt.c",
    "src/lib/container/map.c",
    "src/lib/container/namemap.c",
    "src/lib/container/order.c",
    "src/lib/container/smartlist.c",
    "src/lib/net/address.c",
    "src/lib/net/alertsock.c",
    "src/lib/net/buffers_net.c",
    "src/lib/net/gethostname.c",
    "src/lib/net/inaddr.c",
    "src/lib/net/network_sys.c",
    "src/lib/net/resolve.c",
    "src/lib/net/socket.c",
    "src/lib/net/socketpair.c",
    "src/lib/thread/compat_threads.c",
    "src/lib/thread/numcpus.c",
    "src/lib/thread/compat_pthreads.c",
    "src/lib/memarea/memarea.c",
    "src/lib/math/fp.c",
    "src/lib/math/laplace.c",
    "src/lib/math/prob_distr.c",
    "src/lib/meminfo/meminfo.c",
    "src/lib/osinfo/uname.c",
    "src/lib/log/escape.c",
    "src/lib/log/ratelim.c",
    "src/lib/log/log.c",
    "src/lib/log/log_sys.c",
    "src/lib/log/util_bug.c",
    "src/lib/lock/compat_mutex.c",
    "src/lib/fdio/fdio.c",
    "src/lib/lock/compat_mutex_pthreads.c",
    "src/lib/string/compat_ctype.c",
    "src/lib/string/compat_string.c",
    "src/lib/string/util_string.c",
    "src/lib/string/parse_int.c",
    "src/lib/string/printf.c",
    "src/lib/string/scanf.c",
    "src/lib/term/getpass.c",
    "src/ext/readpassphrase.c",
    "src/lib/smartlist_core/smartlist_core.c",
    "src/lib/smartlist_core/smartlist_split.c",
    "src/lib/malloc/malloc.c",
    "src/lib/malloc/map_anon.c",
    "src/lib/wallclock/approx_time.c",
    "src/lib/wallclock/time_to_tm.c",
    "src/lib/wallclock/tor_gettimeofday.c",
    "src/lib/err/backtrace.c",
    "src/lib/err/torerr.c",
    "src/lib/err/torerr_sys.c",
    "src/lib/version/git_revision.c",
    "src/lib/version/version.c",
    "src/lib/intmath/addsub.c",
    "src/lib/intmath/bits.c",
    "src/lib/intmath/muldiv.c",
    "src/lib/intmath/weakrng.c",
    "src/lib/ctime/di_ops.c",
    "src/ext/trunnel/trunnel.c",
    "src/trunnel/ed25519_cert.c",
    "src/trunnel/link_handshake.c",
    "src/trunnel/pwbox.c",
    "src/trunnel/hs/cell_common.c",
    "src/trunnel/hs/cell_establish_intro.c",
    "src/trunnel/hs/cell_introduce1.c",
    "src/trunnel/hs/cell_rendezvous.c",
    "src/trunnel/channelpadding_negotiation.c",
    "src/trunnel/sendme_cell.c",
    "src/trunnel/socks5.c",
    "src/trunnel/netinfo.c",
    "src/trunnel/circpad_negotiation.c",
    "src/lib/trace/trace.c",
];
