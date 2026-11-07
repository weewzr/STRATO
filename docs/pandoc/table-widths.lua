-- Pipe tables whose rows are wider than the Markdown line limit get equal column widths from
-- pandoc, which wastes space on short columns (symbol, unit, section) and squeezes the long text
-- column. This filter sets the widths in proportion to the text in each column, but only for tables
-- that contain a long text cell and whose widths are still all equal; a table with hand-tuned dash
-- lengths is left alone. Formatting only: no cell content changes.

-- Approximate printed width of a cell in characters: TeX commands count as one character,
-- code counts a little wider than prose.
local function len_of(cell)
  local n = #pandoc.utils.stringify(cell.contents)
  local chunk = 0
  pandoc.Blocks(cell.contents):walk({
    Math = function(m)
      local s = m.text:gsub("\\%a+", "x"):gsub("[{}^_ ]", "")
      n = n - #m.text + #s
    end,
    Code = function(c)
      n = n + math.floor(0.4 * #c.text)
      -- longest piece between the places where break-long-tokens.lua allows a break
      for piece in c.text:gmatch("[^_/:,%.%s%-=]*[_/:,%.%-=]?") do
        if #piece > chunk then chunk = #piece end
      end
    end,
  })
  -- longest piece that TeX cannot break: a word, or a math string up to a times or dot sign
  local buf = {}
  pandoc.Blocks(cell.contents):walk({
    Str = function(s) buf[#buf + 1] = s.text end,
    Space = function() buf[#buf + 1] = " " end,
    SoftBreak = function() buf[#buf + 1] = " " end,
    Math = function(m)
      local s = m.text:gsub("\\times", " "):gsub("\\cdot", " "):gsub("\\%a+", "x"):gsub("[{}^_]", "")
      buf[#buf + 1] = s
    end,
    Code = function(c) buf[#buf + 1] = c.text end,
  })
  local piece = 0
  for word in table.concat(buf):gmatch("%S+") do
    if #word > piece then piece = #word end
  end
  return n, chunk, piece
end

local function TableCore(tbl)
  local specs = tbl.colspecs
  local ncol = #specs
  if ncol < 3 then return nil end
  local first = specs[1][2]
  if not first or first == 0 then return nil end
  for c = 2, ncol do
    local w = specs[c][2]
    if not w or math.abs(w - first) > 1e-6 then return nil end
  end

  local maxlen, sumlen, maxchunk, maxpiece, count = {}, {}, {}, {}, 0
  for c = 1, ncol do maxlen[c] = 0; sumlen[c] = 0; maxchunk[c] = 0; maxpiece[c] = 0 end
  local function scan(rows)
    for _, row in ipairs(rows) do
      count = count + 1
      for c, cell in ipairs(row.cells) do
        if c <= ncol then
          local n, chunk, piece = len_of(cell)
          if piece > maxpiece[c] then maxpiece[c] = piece end
          if chunk > maxchunk[c] then maxchunk[c] = chunk end
          if n > maxlen[c] then maxlen[c] = n end
          sumlen[c] = sumlen[c] + n
        end
      end
    end
  end
  -- body-only measures, used by the numeric-table mode below
  local bodymax, headword = {}, {}
  for c = 1, ncol do bodymax[c] = 0; headword[c] = 0 end
  scan(tbl.head.rows)
  for _, row in ipairs(tbl.head.rows) do
    for c, cell in ipairs(row.cells) do
      if c <= ncol then
        for word in pandoc.utils.stringify(cell.contents):gmatch("%S+") do
          if #word > headword[c] then headword[c] = #word end
        end
      end
    end
  end
  for _, body in ipairs(tbl.bodies) do
    scan(body.body)
    for _, row in ipairs(body.body) do
      for c, cell in ipairs(row.cells) do
        if c <= ncol then
          local n = len_of(cell)
          if n > bodymax[c] then bodymax[c] = n end
        end
      end
    end
  end
  if count == 0 then return nil end
  local longest = 0
  for c = 1, ncol do if maxlen[c] > longest then longest = maxlen[c] end end
  if longest <= 60 then
    -- Four or more columns of short cells (numbers): give each column what its widest cell needs.
    if ncol < 4 then return nil end
    local ws, tot = {}, 0
    for c = 1, ncol do
      local w = 1.9 * math.max(bodymax[c], headword[c]) + 4
      if w < 14 then w = 14 end
      ws[c] = w
      tot = tot + w
    end
    local size = "\\scriptsize"
    local pad = 3
    if ncol >= 7 then
      -- A column needs its longest unbreakable piece plus one character (a trailing times sign)
      -- plus the padding on both sides. At script size (8 pt) one character is about 5.3 pt;
      -- if the columns do not fit in 460 pt, 7 pt type (4.64 pt) with 2.5 pt padding is used.
      local pieces, fulls = {}, {}
      for c = 1, ncol do
        local full = math.max(bodymax[c], headword[c])
        fulls[c] = full
        pieces[c] = math.min(math.max(maxpiece[c], headword[c]), full) + 1
      end
      local function total_need(cw, pd)
        local t = 0
        for c = 1, ncol do t = t + cw * pieces[c] + 2 * pd end
        return t
      end
      local cw = 5.3
      if total_need(cw, pad) > 460 then
        cw, pad, size = 4.64, 2.5, "\\fontsize{7}{8.4}\\selectfont"
      end
      local need, sumneed, extra, sumextra = {}, 0, {}, 0
      for c = 1, ncol do
        need[c] = cw * pieces[c] + 2 * pad
        extra[c] = math.max(fulls[c] + 1 - pieces[c], 0)
        sumneed = sumneed + need[c]
        sumextra = sumextra + extra[c]
      end
      local slack = math.max(460 - sumneed, 0)
      for c = 1, ncol do
        ws[c] = need[c] + (sumextra > 0 and slack * extra[c] / sumextra or slack / ncol)
      end
      tot = 0
      for c = 1, ncol do tot = tot + ws[c] end
    end
    for c = 1, ncol do specs[c][2] = ws[c] / tot end
    tbl.colspecs = specs
    if ncol >= 7 then
      -- Seven or more columns do not fit at footnote size: use a smaller size and tight padding
      -- for this table only (the group ends after the table).
      return {
        pandoc.RawBlock("latex", "\\begingroup\\renewcommand{\\tablefont}{" .. size .. "}\\setlength{\\tabcolsep}{" .. pad .. "pt}"),
        tbl,
        pandoc.RawBlock("latex", "\\endgroup"),
      }
    end
    return tbl
  end

  local weights, total = {}, 0
  for c = 1, ncol do
    local avg = sumlen[c] / count
    local w = 0.55 * math.min(maxlen[c], 50) + 0.45 * math.min(avg, 50)
    local need = 1.3 * math.min(maxlen[c], 16)
    if w < need then w = need end
    -- a code piece that cannot break must fit inside the column (monospace, footnote size)
    local fit = math.min(3.0 * maxchunk[c] + 6, 60)
    if maxchunk[c] > 0 and w < fit then w = fit end
    if w < 14 then w = 14 end
    weights[c] = w
    total = total + w
  end
  for c = 1, ncol do
    specs[c][2] = weights[c] / total
  end
  tbl.colspecs = specs
  return tbl
end

-- In the Final Deliverable the tables carry tall math (integrals, fractions) in narrow cells, so
-- their rows get a little more vertical room. The Full Derivation keeps the default spacing.
local FINAL = false

-- true when a cell holds tall inline math (fraction, sum, integral, floor, root)
local function has_tall_math(tbl)
  local tall = false
  local function scan(rows)
    for _, row in ipairs(rows) do
      for _, cell in ipairs(row.cells) do
        pandoc.Blocks(cell.contents):walk({
          Math = function(m)
            if m.text:find("\\[dt]?frac") or m.text:find("\\sum") or m.text:find("\\int")
                or m.text:find("\\lfloor") or m.text:find("\\sqrt") then
              tall = true
            end
          end,
        })
      end
    end
  end
  scan(tbl.head.rows)
  for _, body in ipairs(tbl.bodies) do scan(body.body) end
  return tall
end

local function Table(tbl)
  local tall = has_tall_math(tbl)
  local res = TableCore(tbl) or tbl
  if not FINAL then return res end
  local out = pandoc.Blocks({})
  local stretch = tall and "1.5" or "1.18"
  out:insert(pandoc.RawBlock("latex", "\\begingroup\\renewcommand{\\tablestretch}{" .. stretch .. "}\\setlength{\\normallineskiplimit}{3pt}\\setlength{\\normallineskip}{3pt}"))
  if res.t == "Table" then
    out:insert(res)
  else
    for _, b in ipairs(res) do out:insert(b) end
  end
  out:insert(pandoc.RawBlock("latex", "\\endgroup"))
  return out
end

return {
  { Meta = function(m) FINAL = pandoc.utils.stringify(m.title or ""):find("Final Deliverable", 1, true) ~= nil end },
  { Table = Table },
}
