import { useState, useEffect, useMemo, useRef } from 'react';
import { useAtomValue, useSetAtom } from 'jotai';
import { templatesAtom, TemplateValues } from './atoms';
import type { StitchTemplate, TemplateField } from './types.gen';

interface TemplatePickerProps {
  projectName: string;
  onTemplateSelect: (template: StitchTemplate | null) => void;
  onValuesChange: (values: TemplateValues) => void;
  selectedTemplate: StitchTemplate | null;
}

// Fetch templates for a project
async function fetchTemplates(project: string): Promise<StitchTemplate[]> {
  const res = await fetch(`/api/p/${encodeURIComponent(project)}/templates`);
  if (!res.ok) throw new Error(`Failed to fetch templates: ${res.status}`);
  return res.json();
}

export default function TemplatePicker({ projectName, onTemplateSelect, onValuesChange, selectedTemplate }: TemplatePickerProps) {
  const setTemplates = useSetAtom(templatesAtom);
  const templates = useAtomValue(templatesAtom);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);

  // Fetch templates on mount if not cached
  useEffect(() => {
    if (templates.length > 0) return; // Already cached
    fetchTemplates(projectName)
      .then(setTemplates)
      .catch(err => console.error('Failed to load templates:', err));
  }, [projectName, templates.length, setTemplates]);

  // Filter templates by search
  const filtered = useMemo(() => {
    if (!search.trim()) return templates;
    const q = search.toLowerCase();
    return templates.filter(t =>
      t.name.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q) ||
      (t.labels && t.labels.some(l => l.toLowerCase().includes(q)))
    );
  }, [templates, search]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  // Clear selection
  const handleClear = () => {
    onTemplateSelect(null);
    onValuesChange({});
  };

  return (
    <div className="template-picker" ref={containerRef}>
      {/* Template selector dropdown */}
      <div className="bdf-field">
        <label className="bdf-label" htmlFor="template-search">
          Template
          {selectedTemplate && <span className="template-selected-badge">selected</span>}
        </label>
        <div className="template-input-row">
          {selectedTemplate ? (
            <div className="template-selected">
              <span className="template-name">{selectedTemplate.name}</span>
              <span className="template-description">{selectedTemplate.description}</span>
              <button
                type="button"
                className="template-clear-btn"
                onClick={handleClear}
                aria-label="Clear template"
              >
                ×
              </button>
            </div>
          ) : (
            <>
              <input
                id="template-search"
                type="text"
                className="bdf-input"
                value={search}
                onChange={e => { setSearch(e.target.value); setOpen(true); }}
                onFocus={() => setOpen(true)}
                placeholder="Search templates…"
              />
            </>
          )}
        </div>

        {/* Template dropdown */}
        {open && !selectedTemplate && (
          <div className="template-dropdown" role="listbox">
            {filtered.length === 0 ? (
              <div className="template-dropdown-empty">No templates found</div>
            ) : (
              filtered.map(t => (
                <div
                  key={t.name}
                  className="template-dropdown-item"
                  role="option"
                  onMouseDown={e => { e.preventDefault(); onTemplateSelect(t); setOpen(false); setSearch(''); }}
                >
                  <div className="template-item-header">
                    <span className="template-item-name">{t.name}</span>
                    {t.scope !== 'global' && (
                      <span className="template-item-scope">{t.scope}</span>
                    )}
                  </div>
                  <div className="template-item-description">{t.description}</div>
                  {(t.kind || (t.labels && t.labels.length > 0)) && (
                    <div className="template-item-meta">
                      {t.kind && <span className="template-item-kind">{t.kind}</span>}
                      {t.labels?.map(l => (
                        <span key={l} className="template-item-label">{l}</span>
                      ))}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}
      </div>

      {/* Template field inputs */}
      {selectedTemplate && selectedTemplate.fields.length > 0 && (
        <TemplateFieldInputs
          fields={selectedTemplate.fields}
          onChange={onValuesChange}
        />
      )}
    </div>
  );
}

// Field inputs for a selected template
interface TemplateFieldInputsProps {
  fields: TemplateField[];
  onChange: (values: TemplateValues) => void;
}

function TemplateFieldInputs({ fields, onChange }: TemplateFieldInputsProps) {
  const [values, setValues] = useState<TemplateValues>(() => {
    const initial: TemplateValues = {};
    for (const field of fields) {
      if (field.default) {
        initial[field.key] = field.default;
      }
    }
    return initial;
  });

  const handleChange = (key: string, value: string) => {
    const updated = { ...values, [key]: value };
    setValues(updated);
    onChange(updated);
  };

  return (
    <div className="template-fields">
      {fields.map(field => (
        <div key={field.key} className="bdf-field">
          <label className="bdf-label" htmlFor={`field-${field.key}`}>
            {field.label}
            {field.required && <span className="bdf-required" aria-hidden>*</span>}
          </label>
          <input
            id={`field-${field.key}`}
            type="text"
            className="bdf-input"
            value={values[field.key] || ''}
            onChange={e => handleChange(field.key, e.target.value)}
            placeholder={field.placeholder || undefined}
            required={field.required}
          />
        </div>
      ))}
    </div>
  );
}
