LET $post = (SELECT * FROM post LIMIT 1);

LET $comment = (RELATE user:admin->comment->$post SET content = "This is a comment on a post.");

RELATE user:admin->comment->$comment SET content = "This is a comment on a comment.";