#!/usr/bin/env bash
#
# SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
# SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
#
# SPDX-License-Identifier: MPL-2.0

# wget exits with non-zero has some pages have errors so we can't stop on errors
# set -e

SCRIPT=$(realpath "$0")
SCRIPTPATH=$(dirname "$SCRIPT")

mkdir -p "$SCRIPTPATH/static_files/"

wget --page-requisites --convert-links --mirror --adjust-extension --directory-prefix="$SCRIPTPATH/static_files/" https://servo.org/
wget --page-requisites --convert-links --mirror --adjust-extension --directory-prefix="$SCRIPTPATH/static_files/" https://www.rust-lang.org/
