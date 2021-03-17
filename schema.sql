CREATE TABLE "repository" (
    id uuid PRIMARY KEY NOT NULL,
    url varchar(255) UNIQUE NOT NULL,
    created_at timestamp NOT NULL DEFAULT NOW(),
    updated_at timestamp NOT NULL DEFAULT NOW(),
    uploader_ip varchar(21) NOT NULL
);

CREATE TABLE "email"(
    email varchar(120) PRIMARY KEY NOT NULL,
    hash_md5 varchar(32) UNIQUE NOT NULL
);

CREATE TABLE "commit" (
    hash varchar(40) PRIMARY KEY NOT NULL,
    tree varchar(40) REFERENCES commit(hash) ON DELETE CASCADE NULL,
    text text NOT NULL,
    date timestamptz NOT NULL,
    author_email varchar(120) REFERENCES email(email) ON DELETE NO ACTION NOT NULL,
    author_name varchar(120) NOT NULL,
    committer_email varchar(120) REFERENCES email(email) ON DELETE NO ACTION NOT NULL,
    committer_name varchar(120) NOT NULL,
    repository_url varchar(256) REFERENCES repository(url) ON DELETE CASCADE NOT NULL
);

CREATE TABLE "branch" (
    id uuid PRIMARY KEY NOT NULL,
    name varchar(120) NOT NULL,
    repository_id uuid REFERENCES repository(id) ON DELETE CASCADE NOT NULL,
    head varchar(40) REFERENCES commit(hash) ON DELETE SET NULL NULL
);
