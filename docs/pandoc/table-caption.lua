-- Table captions are written in Markdown as "Table: 5.1 Caption." so they read well on GitHub.
-- LaTeX automatic caption numbers are switched off (see defaults.yaml), so this filter adds the
-- word "Table" in front of a caption that starts with a manual number such as "5.1".
function Table(tbl)
  local first = tbl.caption.long[1]
  if first and first.content and first.content[1] and first.content[1].t == "Str" then
    local s = first.content[1].text
    if s:match("^%d+%.%d+") or s:match("^%u%.%d+") then
      first.content[1].text = "Table " .. s
    end
  end
  return tbl
end
