SELECT DISTINCT f.path FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ({include_placeholders})
{exclude_clause}
GROUP BY f.id
{having_clause}
