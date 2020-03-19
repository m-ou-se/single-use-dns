use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use tokio::net::{TcpListener, UdpSocket};
use tokio::runtime;
use trust_dns_client::rr::{RecordSet, RecordType, RrKey};
use trust_dns_server::authority::{Catalog, ZoneType};
use trust_dns_server::proto::rr::rdata::{SOA, TXT};
use trust_dns_server::proto::rr::record_data::RData;
use trust_dns_server::proto::rr::resource::Record;
use trust_dns_server::proto::rr::Name;
use trust_dns_server::store::in_memory::InMemoryAuthority;
use trust_dns_server::ServerFuture;

/// Serves DNS records for a single domain name.
///
/// Useful to temporarily host a DNS record, such as a ACME DNS-01 challenge.
#[derive(StructOpt)]
#[structopt(setting = AppSettings::ColorAuto)]
#[structopt(setting = AppSettings::UnifiedHelpMessage)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
struct Args {
	/// Domain name to serve records for.
	#[structopt(long = "domain", short, parse(try_from_str))]
	dns: Name,

	/// A record to serve.
	#[structopt(short, long, parse(try_from_str), number_of_values = 1)]
	a: Vec<Ipv4Addr>,

	/// AAAA record to serve.
	#[structopt(long, parse(try_from_str), number_of_values = 1)]
	aaaa: Vec<Ipv6Addr>,

	/// TXT record to serve.
	#[structopt(long, number_of_values = 1)]
	txt: Vec<String>,

	/// The address(es) and port(s) to listen on.
	#[structopt(long, short, default_value = "[::]:53", use_delimiter = true)]
	listen: Vec<SocketAddr>,
}

fn main() {
	let args = Args::from_args();

	let mut records = BTreeMap::new();

	records.insert(
		RrKey::new(args.dns.clone().into(), RecordType::SOA),
		Record::from_rdata(
			args.dns.clone(),
			0,
			RData::SOA(SOA::new(args.dns.clone(), args.dns.clone(), 0, 0, 0, 0, 0)),
		)
		.into(),
	);

	let mut n_records = 0;

	if !args.a.is_empty() {
		let mut recordset = RecordSet::new(&args.dns, RecordType::A, 0);
		for a in args.a {
			recordset.add_rdata(RData::A(a));
			n_records += 1;
		}
		records.insert(
			RrKey::new(args.dns.clone().into(), RecordType::A),
			recordset,
		);
	}

	if !args.aaaa.is_empty() {
		let mut recordset = RecordSet::new(&args.dns, RecordType::AAAA, 0);
		for aaaa in args.aaaa {
			recordset.add_rdata(RData::AAAA(aaaa));
			n_records += 1;
		}
		records.insert(
			RrKey::new(args.dns.clone().into(), RecordType::AAAA),
			recordset,
		);
	}

	if !args.txt.is_empty() {
		let mut recordset = RecordSet::new(&args.dns, RecordType::TXT, 0);
		for txt in args.txt {
			recordset.add_rdata(RData::TXT(TXT::new(vec![txt])));
			n_records += 1;
		}
		records.insert(
			RrKey::new(args.dns.clone().into(), RecordType::TXT),
			recordset,
		);
	}

	let authority =
		InMemoryAuthority::new(args.dns.clone(), records, ZoneType::Master, false).unwrap();

	let mut catalog = Catalog::new();

	catalog.upsert(args.dns.clone().into(), Box::new(authority));

	let mut server = ServerFuture::new(catalog);

	let mut runtime = runtime::Builder::new()
		.enable_all()
		.basic_scheduler()
		.core_threads(1)
		.build()
		.unwrap();

	for addr in args.listen {
		let udp_socket = runtime
			.block_on(UdpSocket::bind(addr))
			.unwrap_or_else(|_| panic!("Could not bind to UDP: {:?}", addr));
		server.register_socket(udp_socket, &runtime);
		let tcp_listener = runtime
			.block_on(TcpListener::bind(addr))
			.unwrap_or_else(|_| panic!("Could not bind to TCP: {:?}", addr));
		server
			.register_listener(tcp_listener, Duration::from_secs(5), &runtime)
			.unwrap();
		eprintln!("Listening on {} (UDP and TCP)", addr);
	}

	eprintln!("Serving {} record(s) for {}", n_records, args.dns);

	runtime.block_on(server.block_until_done()).unwrap();
}
