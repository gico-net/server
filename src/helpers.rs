use regex::Regex;
use uuid::Uuid;

/// Returns a valid Uuid if `id` is not a valid Uuid
pub fn uuid_from_string(id: &String) -> Uuid {
    return match Uuid::parse_str(&id) {
        Ok(x) => x,
        Err(_) => Uuid::parse_str("00000000000000000000000000000000").unwrap(),
    };
}

/// Check if a path is into the "valid git repositories" and returns the name
pub fn name_of_git_repository(url: &String) -> Option<String> {
    const GITHUB_RE: &str = r"^(http(s)?://)?(www.)?github.com/(?P<username>[a-zA-Z0-9-]+)/(?P<repository>[a-zA-Z0-9-]+)";
    let re = Regex::new(GITHUB_RE).unwrap();

    if !re.is_match(&url) {
        return None;
    }

    let captures = re.captures(&url).unwrap();
    let name = captures.name("username").unwrap().as_str();
    let repo = captures.name("repository").unwrap().as_str();

    Some(format!("{}/{}", name, repo))
}
