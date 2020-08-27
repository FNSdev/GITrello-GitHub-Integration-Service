create table board_repository (
    id serial primary key,
    board_id bigint not null,
    repository_id bigint not null,
    github_profile_id int not null,
    constraint fk_github_profile
        foreign key (github_profile_id)
            references github_profile(id)
            on delete cascade,
    constraint uq_board_repository
        unique (board_id, repository_id)
);
