;; -*- no-byte-compile: t; -*-
;;; packages.el --- DevX AI completion packages

(package! copilot
  :recipe (:host github
           :repo "copilot-emacs/copilot.el"
           :files ("*.el"))
  :pin "90f429b100418d897b673d6c36c2817364b28e3e")
(package! tabbymacs
  :recipe (:host github
           :repo "Bastillan/tabbymacs"
           :files ("*.el"))
  :pin "32b8c40774515c940a02c78874cf928b356478de")
