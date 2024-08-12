DEFINE TABLE OVERWRITE user SCHEMALESS;

DEFINE FIELD OVERWRITE username ON user TYPE string;
DEFINE FIELD OVERWRITE email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD OVERWRITE password ON user TYPE string;
DEFINE FIELD OVERWRITE registered_at ON user TYPE datetime DEFAULT time::now();