DELETE FROM file_tags
WHERE file_id IN (SELECT id FROM files WHERE path = ?1)
AND tag_id IN (SELECT id FROM tags WHERE name = ?2)
