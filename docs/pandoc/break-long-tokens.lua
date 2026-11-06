-- Gives LaTeX a place to break long unbroken tokens (file paths, identifiers, DOI links) so that
-- they do not run past the right margin. Formatting only: the visible text is unchanged.

local ESC = {
  ["\\"] = "\\textbackslash{}", ["{"] = "\\{", ["}"] = "\\}", ["$"] = "\\$", ["&"] = "\\&",
  ["#"] = "\\#", ["%"] = "\\%", ["_"] = "\\_", ["^"] = "\\^{}", ["~"] = "\\~{}",
  ["'"] = "\\textquotesingle{}", ["`"] = "\\textasciigrave{}",
}
local BREAK_AFTER = { ["/"] = true, ["_"] = true, [":"] = true, ["="] = true, [","] = true }
local BREAK_AFTER_BEFORE_LETTER = { ["."] = true, ["-"] = true }

-- Plain inline code (no language class): emit \texttt with \allowbreak after path-like characters.
function Code(el)
  if #el.classes > 0 then return nil end
  local chars = {}
  for ch in el.text:gmatch(utf8.charpattern) do chars[#chars + 1] = ch end
  local out = {}
  local prev_space = false
  for i, ch in ipairs(chars) do
    local nxt = chars[i + 1] or ""
    if ch == " " then
      out[#out + 1] = prev_space and "\\ " or " "
      prev_space = true
    else
      prev_space = false
      out[#out + 1] = ESC[ch] or ch
      if BREAK_AFTER[ch] or (BREAK_AFTER_BEFORE_LETTER[ch] and nxt:match("^%a")) then
        out[#out + 1] = "\\allowbreak{}"
      end
    end
  end
  return pandoc.RawInline("latex", "\\texttt{" .. table.concat(out) .. "}")
end

-- Plain text tokens: web addresses become \url (xurl breaks them); long path-like words get
-- break points after "/" and "_".
function Str(el)
  local t = el.text
  if t:find("://", 1, true) then
    local url, tail = t:match("^(.-)([%.,;:%)]*)$")
    local res = { pandoc.RawInline("latex", "\\url{" .. url .. "}") }
    if tail ~= "" then res[#res + 1] = pandoc.Str(tail) end
    return res
  end
  if #t >= 14 and t:find("[/_]") then
    local out, i = {}, 1
    while i <= #t do
      local j = t:find("[/_]", i)
      if (not j) or j == #t then
        out[#out + 1] = pandoc.Str(t:sub(i))
        break
      end
      out[#out + 1] = pandoc.Str(t:sub(i, j))
      out[#out + 1] = pandoc.RawInline("latex", "\\allowbreak{}")
      i = j + 1
    end
    return out
  end
  return nil
end
