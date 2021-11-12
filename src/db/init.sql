CREATE TABLE access_info (
    id INTEGER PRIMARY KEY ASC AUTOINCREMENT,
    time TEXT,
    src_port INT,
    src_ip TEXT,
    dst_port INT,
    dst_domain TEXT,
    state TEXT,
    protocol TEXT
);
