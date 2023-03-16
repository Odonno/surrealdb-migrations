DEFINE TABLE user SCHEMALESS;

DEFINE FIELD username ON TABLE user TYPE string ASSERT $value != NONE;
DEFINE FIELD email ON TABLE user TYPE string ASSERT is::email($value);
DEFINE FIELD password ON TABLE user TYPE string ASSERT $value != NONE;
DEFINE FIELD registered_at ON user TYPE datetime VALUE $before OR time::now();