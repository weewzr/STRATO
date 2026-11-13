-- Keeps a heading together with a table that follows it directly, and a paragraph that is only a
-- bold run-in label (for example "Release fractions by scenario.") together with what follows,
-- so neither is left alone at the bottom of a page. A label followed by a code block is handled
-- by keep-code-together.lua. Formatting only: no content changes.

local function bold_label_only(block)
  return block.t == "Para" and #block.content == 1 and block.content[1].t == "Strong"
end

function Blocks(blocks)
  local out = pandoc.Blocks({})
  for i, b in ipairs(blocks) do
    local nxt = blocks[i + 1]
    if b.t == "Header" and nxt and nxt.t == "Table" then
      out:insert(pandoc.RawBlock("latex", "\\Needspace{16\\baselineskip}"))
    elseif bold_label_only(b) and nxt and nxt.t ~= "CodeBlock" then
      out:insert(pandoc.RawBlock("latex", "\\Needspace{6\\baselineskip}"))
    end
    out:insert(b)
  end
  return out
end
