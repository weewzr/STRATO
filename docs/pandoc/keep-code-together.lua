-- Keeps a bold label such as "Pseudocode." or "Rust." together with the short code block that
-- follows it: if the block does not fit in the rest of the page, the label and the block move to
-- the next page. Blocks of more than MAX_LINES lines are allowed to split. The label "Final
-- equation." is kept with the display equation that follows it. Formatting only.

local MAX_LINES = 42
local LABELS = { ["Pseudocode."] = true, ["Rust."] = true, ["Final equation."] = true }

local function is_label(block)
  if block.t ~= "Para" and block.t ~= "Plain" then return false end
  if #block.content ~= 1 or block.content[1].t ~= "Strong" then return false end
  return LABELS[pandoc.utils.stringify(block.content[1])] == true
end

local function has_display_math(block)
  if block.t ~= "Para" and block.t ~= "Plain" then return false end
  for _, inl in ipairs(block.content) do
    if inl.t == "Math" and inl.mathtype == "DisplayMath" then return true end
  end
  return false
end

function Blocks(blocks)
  local out = pandoc.Blocks({})
  local i = 1
  while i <= #blocks do
    local b = blocks[i]
    local nxt = blocks[i + 1]
    -- a section heading followed by an "Operation B.n." paragraph needs room for the paragraph,
    -- the "Final equation." label and the first equation
    if b.t == "Header" and nxt and nxt.t == "Para" and #nxt.content > 0
        and nxt.content[1].t == "Strong"
        and pandoc.utils.stringify(nxt.content[1]):match("^Operation B%.") then
      out:insert(pandoc.RawBlock("latex", "\\Needspace{14\\baselineskip}"))
    end
    if is_label(b) and nxt then
      if nxt.t == "CodeBlock" then
        local n = select(2, nxt.text:gsub("\n", "\n")) + 1
        if n <= MAX_LINES then
          local lines = math.ceil(0.85 * n) + 3
          out:insert(pandoc.RawBlock("latex", "\\Needspace*{" .. lines .. "\\baselineskip}"))
        end
      elseif has_display_math(nxt) then
        out:insert(pandoc.RawBlock("latex", "\\Needspace*{9\\baselineskip}"))
      end
    end
    out:insert(b)
    i = i + 1
  end
  return out
end
