#pragma once

#include <qsgrendernode.h>
#include <QQuickItem>

#if QT_CONFIG(opengl)

QT_BEGIN_NAMESPACE

class QOpenGLShaderProgram;
class QOpenGLBuffer;

QT_END_NAMESPACE

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
};

#endif // opengl