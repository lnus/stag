SELECT t.name
FROM tags t
JOIN file_tags ft ON t.id = ft.tag_id
JOIN files f ON f.id = ft.file_id
WHERE f.path = ?1
