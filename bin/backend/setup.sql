create table users (
    id uuid primary key,
    username varchar not null,
    hashed_password varchar not null,
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null
);

create unique index users_username on users(username);

create table auth_tokens (
    id uuid primary key,
    user_id uuid not null references users (id),
    token varchar not null,
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null
);

create unique index auth_tokens_token on auth_tokens(token);

create table tweets (
    id uuid primary key,
    user_id uuid not null references users (id),
    text text not null,
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null
);

create table follows (
    id uuid primary key,
    follower_id uuid not null references users (id),
    followee_id uuid not null references users (id),
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null
);

create unique index follows_follower_followee on follows(follower_id, followee_id);
