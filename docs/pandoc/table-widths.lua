-- Pipe tables whose rows are wider than the Markdown line limit get equal column widths from
-- pandoc, which wastes space on short columns (symbol, unit, section) and squeezes the long text
-- column. This filter sets the widths in proportion to the text in each column, but only for tables
-- whose widths are still all equal; a table with hand-tuned dash lengths is left alone.
-- Formatting only: no cell content changes.

local function len_of(cell)
  return #pandoc.utils.stringify(cell.contents)
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

  local maxlen, sumlen, count = {}, {}, 0
  for c = 1, ncol do maxlen[c] = 0; sumlen[c] = 0 end
  local function scan(rows)
    for _, row in ipairs(rows) do
      count = count + 1
      for c, cell in ipairs(row.cells) do
        if c <= ncol then
          local n = len_of(cell)
          if n > maxlen[c] then maxlen[c] = n end
          sumlen[c] = sumlen[c] + n
        end
      end
    end
  end
  scan(tbl.head.rows)
  for _, body in ipairs(tbl.bodies) do scan(body.body) end
  if count == 0 then return nil end

  local weights, total = {}, 0
  for c = 1, ncol do
    local avg = sumlen[c] / count
    local w = 0.6 * math.min(maxlen[c], 60) + 0.4 * math.min(avg, 60)
    if w < 9 then w = 9 end
    weights[c] = w
    total = total + w
  end
  for c = 1, ncol do
    specs[c][2] = weights[c] / total
  end
  tbl.colspecs = specs
  return tbl
end
