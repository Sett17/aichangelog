use clap::Parser;

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let repo = git2::Repository::open(".").unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_range(&args.range).unwrap();

    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        let message = commit.message().unwrap();
        let message = if args.short {
            message.lines().next().unwrap()
        } else {
            message
        };
        println!("{} {}", commit.id(), message);
    }
}

// tool to generate changelog from commit range
// ranges:
//   hash to head
//   hash to hash
//   tag to head
//   tag to tag
//   tag to hash

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///Rev range to generate changelog from
    range: String,

    ///Only use first line of commit message to reduce tokens
    #[arg(short, long)]
    short: bool,
}
