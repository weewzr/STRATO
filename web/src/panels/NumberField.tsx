import { useId, useState, type ReactNode } from 'react';
import { plain } from '../format';
import { parseNumber } from '../state/validation';

interface Props {
  label: ReactNode;
  unit?: ReactNode;
  value: number;
  onCommit: (v: number) => void;
  /** Message from the input checks for this field, if any. */
  issue?: string | undefined;
  hint?: ReactNode;
  integer?: boolean;
  testId?: string;
}

/**
 * Text field for a number in plain or scientific notation. A typed value is sent to the store as
 * soon as it parses; text that does not parse stays in the field, is marked invalid and is not sent.
 */
export function NumberField({ label, unit, value, onCommit, issue, hint, integer = false, testId }: Props) {
  const id = useId();
  const [draft, setDraft] = useState(() => plain(value));
  const [seen, setSeen] = useState(value);
  if (!Object.is(seen, value)) {
    // The store changed from elsewhere (reset, case load): show the new value.
    setSeen(value);
    setDraft(plain(value));
  }
  const parsed = parseNumber(draft);
  const parseError = parsed === null ? 'Not a number. Use digits, a decimal point and the letter e before a power of ten.' : undefined;
  const intError = parsed !== null && integer && !Number.isInteger(parsed) ? 'Must be a whole number.' : undefined;
  const message = parseError ?? intError ?? issue;
  const describedBy = [hint ? `${id}-hint` : '', message ? `${id}-msg` : ''].filter(Boolean).join(' ') || undefined;

  return (
    <div className="field">
      <label htmlFor={id} className="field__label">
        {label}
        {unit ? <span className="field__unit"> ({unit})</span> : null}
      </label>
      <input
        id={id}
        className="field__input"
        type="text"
        inputMode={integer ? 'numeric' : 'decimal'}
        autoComplete="off"
        spellCheck={false}
        value={draft}
        aria-invalid={message ? true : undefined}
        aria-describedby={describedBy}
        data-testid={testId}
        onChange={(e) => {
          const text = e.target.value;
          setDraft(text);
          const v = parseNumber(text);
          if (v !== null && (!integer || Number.isInteger(v))) {
            setSeen(v);
            onCommit(v);
          }
        }}
        onBlur={() => {
          if (parsed !== null && !parseError && !intError) setDraft(plain(parsed));
        }}
      />
      {hint ? (
        <p id={`${id}-hint`} className="field__hint">
          {hint}
        </p>
      ) : null}
      {message ? (
        <p id={`${id}-msg`} className="field__msg" role="status">
          {message}
        </p>
      ) : null}
    </div>
  );
}
