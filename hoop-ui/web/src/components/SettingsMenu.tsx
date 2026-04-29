import { useAtom } from 'jotai';
import { useEffect, useCallback } from 'react';
import { settingsMenuOpenAtom } from '../atoms';
import { WelcomeTourTrigger } from './WelcomeTour';
import { useOnboarding } from '../useOnboarding';

export function SettingsMenu() {
  const [isOpen, setIsOpen] = useAtom(settingsMenuOpenAtom);
  const { promptsResponse, setPromptsEnabled } = useOnboarding();

  const handleOpen = useCallback(() => {
    setIsOpen(true);
  }, [setIsOpen]);

  const handleClose = useCallback(() => {
    setIsOpen(false);
  }, [setIsOpen]);

  // Listen for open-settings event
  useEffect(() => {
    const handleOpenSettings = () => setIsOpen(true);
    window.addEventListener('hoop-open-settings', handleOpenSettings);
    return () => window.removeEventListener('hoop-open-settings', handleOpenSettings);
  }, [setIsOpen]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
      }
    };
    if (isOpen) {
      window.addEventListener('keydown', handleEscape);
      return () => window.removeEventListener('keydown', handleEscape);
    }
  }, [isOpen, handleClose]);

  // Close on click outside
  useEffect(() => {
    if (isOpen) {
      const handleClickOutside = (e: MouseEvent) => {
        const target = e.target as HTMLElement;
        if (!target.closest('.settings-menu-container')) {
          handleClose();
        }
      };
      // Use mousedown to catch the click before it completes
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isOpen, handleClose]);

  return (
    <div className="settings-menu-container">
      <button
        className="settings-trigger"
        onClick={handleOpen}
        aria-label="Open settings"
        title="Settings"
      >
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
          <path
            d="M10 3.5C10.4142 3.5 10.75 3.83579 10.75 4.25V5.5H9.25V4.25C9.25 3.83579 9.58579 3.5 10 3.5ZM9.25 14.5V15.75C9.25 16.1642 9.58579 16.5 10 16.5C10.4142 16.5 10.75 16.1642 10.75 15.75V14.5H9.25ZM4.25 9.25H5.5V10.75H4.25C3.83579 10.75 3.5 10.4142 3.5 10C3.5 9.58579 3.83579 9.25 4.25 9.25ZM14.5 10.75H15.75C16.1642 10.75 16.5 10.4142 16.5 10C16.5 9.58579 16.1642 9.25 15.75 9.25H14.5V10.75ZM5.6967 5.6967C5.97205 5.42136 6.41628 5.42136 6.69162 5.6967L7.57452 6.5796C7.84986 6.85494 7.84986 7.29917 7.57452 7.57452C7.29917 7.84986 6.85494 7.84986 6.5796 7.57452L5.6967 6.69162C5.42136 6.41628 5.42136 5.97205 5.6967 5.6967ZM12.4255 12.4255C12.7008 12.1501 13.1451 12.1501 13.4204 12.4255L14.3033 13.3084C14.5786 13.5837 14.5786 14.028 14.3033 14.3033C14.028 14.5786 13.5837 14.5786 13.3084 14.3033L12.4255 13.4204C12.1501 13.1451 12.1501 12.7008 12.4255 12.4255ZM7.57452 12.4255C7.84986 12.7008 7.84986 13.1451 7.57452 13.4204L6.69162 14.3033C6.41628 14.5786 5.97205 14.5786 5.6967 14.3033C5.42136 14.028 5.42136 13.5837 5.6967 13.3084L6.5796 12.4255C6.85494 12.1501 7.29917 12.1501 7.57452 12.4255ZM14.3033 5.6967C14.5786 5.97205 14.5786 6.41628 14.3033 6.69162L13.4204 7.57452C13.1451 7.84986 12.7008 7.84986 12.4255 7.57452C12.1501 7.29917 12.1501 6.85494 12.4255 6.5796L13.3084 5.6967C13.5837 5.42136 14.028 5.42136 14.3033 5.6967ZM10 8C8.89543 8 8 8.89543 8 10C8 11.1046 8.89543 12 10 12C11.1046 12 12 11.1046 12 10C12 8.89543 11.1046 8 10 8ZM6.5 10C6.5 8.067 8.067 6.5 10 6.5C11.933 6.5 13.5 8.067 13.5 10C13.5 11.933 11.933 13.5 10 13.5C8.067 13.5 6.5 11.933 6.5 10Z"
            fill="currentColor"
          />
        </svg>
      </button>

      {isOpen && (
        <div className="settings-menu-overlay" onClick={handleClose}>
          <div
            className="settings-menu-panel"
            onClick={e => e.stopPropagation()}
            role="dialog"
            aria-label="Settings menu"
            aria-modal="true"
          >
            <div className="settings-menu-header">
              <h3>Settings</h3>
              <button
                className="settings-menu-close"
                onClick={handleClose}
                aria-label="Close settings"
              >
                ×
              </button>
            </div>
            <div className="settings-menu-content">
              <div className="settings-menu-section">
                <h4>Onboarding</h4>
                <p className="settings-menu-description">
                  Revisit the welcome tour to learn about HOOP features.
                </p>
                <WelcomeTourTrigger />
              </div>
              <div className="settings-menu-section">
                <h4>Feature Discovery</h4>
                <p className="settings-menu-description">
                  Show tips and suggestions for unused features.
                </p>
                {promptsResponse && (
                  <label className="settings-toggle">
                    <input
                      type="checkbox"
                      checked={promptsResponse.prompts_enabled}
                      onChange={e => {
                        setPromptsEnabled(e.target.checked);
                      }}
                    />
                    <span>Enable prompts</span>
                  </label>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
