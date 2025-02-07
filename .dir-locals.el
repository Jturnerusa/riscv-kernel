((rust-ts-mode . ((eglot-workspace-configuration
                   .
                   (:rust-analyzer
                    (:check
                     (:overrideCommand
                      ["cargo" "clippy" "--message-format=json"]))))
                  (fmt-executable . "rustfmt")
                  (fmt-args . ("--edition=2021"))
                  (eval . (eglot-ensure))
                  (eval . (company-mode 1))
                  (eval . (add-hook 'before-save-hook 'fmt-current-buffer nil t))
                  (eval . (add-hook 'eglot-managed-mode-hook (lambda ()
                                                               (eglot-inlay-hints-mode -1))
                                    nil t)))))
