#pragma once

#include <qsgrendernode.h>
#include <QQuickItem>

#include "rust/cxx.h"

#include <optional>

#if QT_CONFIG(opengl)

QT_BEGIN_NAMESPACE

class QOpenGLShaderProgram;
class QOpenGLBuffer;

QT_END_NAMESPACE

class QServoSwapChain;

// TODO: could instead have the servo engine live in this thread and class
// then sync the Qt options via the sync method?
class QServoGLRenderNode : public QSGRenderNode
{
public:
    ~QServoGLRenderNode();

    void render(const RenderState *state) override;
    void releaseResources() override;
    StateFlags changedStates() const override;
    RenderingFlags flags() const override;
    QRectF rect() const override;

    void sync(QQuickItem *item);

private:
    void init();

    int m_width = 0;
    int m_height = 0;
    std::optional<::rust::Box<QServoSwapChain>> m_swapChain = std::nullopt;
};

#endif // opengl