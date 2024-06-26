// clang-format off
// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// clang-format on
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

#include "helpers.h"

#include <QOpenGLFramebufferObject>
#include <QOpenGLContext>
#include <QOpenGLFunctions>

void
blitFramebuffer(QOpenGLFramebufferObject* target, ::std::unique_ptr<QOpenGLFramebufferObject> source)
{
    Q_ASSERT(target != nullptr);
    Q_ASSERT(source != nullptr);
    QOpenGLFramebufferObject::blitFramebuffer(target, source.get());
}

::std::unique_ptr<QOpenGLFramebufferObject>
fboFromTexture(unsigned int texture_id, unsigned int texture_target, QSize size)
{
    QOpenGLFunctions *f = QOpenGLContext::currentContext()->functions();

    auto fbo = ::std::make_unique<QOpenGLFramebufferObject>(size);
    f->glFramebufferTexture2D(GL_READ_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, texture_target, texture_id, 0);
    Q_ASSERT(fbo->isValid());
    return fbo;
}

::rust::isize
qTouchEventPointCount(QTouchEvent const& event)
{
    return static_cast<::rust::isize>(event.pointCount());
}

QEventPoint const&
qTouchEventPoint(QTouchEvent& event, ::rust::isize index)
{
    return (event.point(static_cast<qsizetype>(index)));
}
