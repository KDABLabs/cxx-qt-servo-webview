// clang-format off
// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// clang-format on
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

#pragma once

#include <memory>

#include <QtCore/QSize>
#include <QtQuick/QQuickFramebufferObject>

class QOpenGLFramebufferObject;

// Alias for CXX, could use custom type in Rust to use type_id
using QQuickFramebufferObjectRenderer = QQuickFramebufferObject::Renderer;

// TODO: useful to add to cxx-qt-lib anyway for opaque types?
template<typename T, typename... Args>
::std::unique_ptr<T> constructUniquePtr(Args... args)
{
    return ::std::make_unique<T>(args...);
}

void
blitFramebuffer(QOpenGLFramebufferObject* target, QOpenGLFramebufferObject* source);

QOpenGLFramebufferObject*
fboFromTexture(unsigned int texture_id, unsigned int texture_target, QSize size);