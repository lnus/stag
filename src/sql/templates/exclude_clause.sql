AND f.id NOT IN (
SELECT f2.id FROM files f2
JOIN file_tags ft2 ON f2.id = ft2.file_id
JOIN tags t2 ON ft2.tag_id = t2.id
WHERE t2.name IN ({exclude_placeholders})
)
