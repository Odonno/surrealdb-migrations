DEFINE TABLE user SCHEMALESS;

DEFINE FIELD username ON user TYPE string;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD password ON user TYPE string;
DEFINE FIELD registered_at ON user TYPE datetime DEFAULT time::now();