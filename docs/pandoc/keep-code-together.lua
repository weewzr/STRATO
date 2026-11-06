-- Keeps a bold label such as "Pseudocode." or "Rust." together with the short code block that
-- follows it: if the block does not fit in the rest of the page, the label and the block move to
-- the next page. Blocks of more than MAX_LINES lines are allowed to split. Formatting only.

local MAX_LINES = 42
local LABELS = { ["Pseudocode."] = true, ["Rust."] = true }

local function is_label(block)
  if block.t ~= "Para" and block.t ~= "Plain" then return false end
  if #block.content ~= 1 or block.content[1].t ~= "Strong" then return false end
  return LABELS[pandoc.utils.stringify(block.content[1])] == true
end

function Blocks(blocks)
  local out = pandoc.Blocks({})
  local i = 1
  while i <= #blocks do
    local b = blocks[i]
    local nxt = blocks[i + 1]
    if is_label(b) and nxt and nxt.t == "CodeBlock" then
      local n = select(2, nxt.text:gsub("\n", "\n")) + 1
      if n <= MAX_LINES then
        local lines = math.ceil(0.85 * n) + 3
        out:insert(pandoc.RawBlock("latex", "\\Needspace*{" .. lines .. "\\baselineskip}"))
      end
    end
    out:insert(b)
    i = i + 1
  end
  return out
end
