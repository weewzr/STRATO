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
  return n, chunk
end

function Table(tbl)
  local specs = tbl.colspecs
  local ncol = #specs
  if ncol < 3 then return nil end
  local first = specs[1][2]
  if not first or first == 0 then return nil end
  for c = 2, ncol do
    local w = specs[c][2]
    if not w or math.abs(w - first) > 1e-6 then return nil end
  end

  local maxlen, sumlen, maxchunk, count = {}, {}, {}, 0
  for c = 1, ncol do maxlen[c] = 0; sumlen[c] = 0; maxchunk[c] = 0 end
  local function scan(rows)
    for _, row in ipairs(rows) do
      count = count + 1
      for c, cell in ipairs(row.cells) do
        if c <= ncol then
          local n, chunk = len_of(cell)
          if chunk > maxchunk[c] then maxchunk[c] = chunk end
          if n > maxlen[c] then maxlen[c] = n end
          sumlen[c] = sumlen[c] + n
        end
      end
    end
  end
  scan(tbl.head.rows)
  for _, body in ipairs(tbl.bodies) do scan(body.body) end
  if count == 0 then return nil end
  local longest = 0
  for c = 1, ncol do if maxlen[c] > longest then longest = maxlen[c] end end
  if longest <= 60 then return nil end

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
