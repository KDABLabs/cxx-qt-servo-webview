// clang-format off
// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// clang-format on
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

#pragma once

#include <memory>

#include <QQuickFramebufferObject>

// Alias for CXX, could use custom type in Rust to use type_id
using QQuickFramebufferObjectRenderer = QQuickFramebufferObject::Renderer;

// Add QObject to QQuickFramebufferObjectRenderer so that CXX-Qt can derive from it
class QQuickFramebufferObjectRendererWithQObject : public QObject, public QQuickFramebufferObjectRenderer {
    Q_OBJECT

public:
    explicit QQuickFramebufferObjectRendererWithQObject();
    ~QQuickFramebufferObjectRendererWithQObject() override;
};

// TODO: useful to add to cxx-qt-lib anyway for opaque types?
template<typename T, typename... Args>
::std::unique_ptr<T> constructUniquePtr(Args... args)
{
    return ::std::make_unique<T>(args...);
}

#include <QSize>

class QOpenGLFramebufferObject;

void
blitFramebuffer(QOpenGLFramebufferObject* target, QOpenGLFramebufferObject* source);

QOpenGLFramebufferObject*
fboFromTexture(unsigned int texture_id, unsigned int texture_target, QSize size);