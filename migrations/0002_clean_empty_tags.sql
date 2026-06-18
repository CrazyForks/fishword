UPDATE cards
SET tags = (
    SELECT COALESCE(json_group_array(value), '[]')
    FROM json_each(tags)
    WHERE value != ''
)
WHERE EXISTS (
    SELECT 1 FROM json_each(tags) WHERE value = ''
);
